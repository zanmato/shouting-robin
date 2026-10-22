//! Groups a crawl's pages into templates by their layout.
//!
//! Structured data names a page first: pages whose most specific type is
//! `Product` form the Product template, as long as each one's layout fits
//! what the others share. Pages without a type, or whose layout fails their
//! type's template, join the template their layout fits. A page without
//! Product markup that joins the Product template is what the Overview
//! reports. Whatever is left is clustered by layout alone.
//!
//! A template is learned from its pages' grouping paths: paths on most pages
//! are core, paths on some are optional regions. Paths on nearly every page
//! of the site (header, footer) are site chrome and ignored throughout.
//!
//! Large crawls are read in batches and templates are learned from a sample
//! of each type's pages, so memory stays bounded by the sample sizes.

use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;

use crate::crawl::layout::{LayoutTables, PageLayout, for_each_layout, page_shape};

/// Share of a site's pages a path or type must appear on to be site-wide.
const SITE_WIDE: f64 = 0.85;
/// Share of a template's pages a path must appear on to be core, or optional.
const CORE: f64 = 0.75;
const OPTIONAL: f64 = 0.25;
/// Membership: the share of the core a page covers, and the share of the page
/// the template explains.
const COVERAGE: f64 = 0.85;
const EXPLAINED: f64 = 0.7;
/// Average linkage cut for grouping by layout alone.
const SHAPE_CUT: f64 = 0.5;
/// Pages per type a template is learned from.
const SAMPLE_LIMIT: usize = 400;
/// Leftover pages clustered by layout. The clustering is quadratic in
/// memory, so pages beyond this join the clusters found instead.
const CLUSTER_LIMIT: usize = 600;

/// The type that names a page, most specific first. Types further down only
/// name a page when nothing above them is present.
const TYPE_PRIORITY: [&str; 24] = [
    "Product",
    "ProductGroup",
    "IndividualProduct",
    "Vehicle",
    "Book",
    "SoftwareApplication",
    "CollectionPage",
    "SearchResultsPage",
    "OfferCatalog",
    "ItemList",
    "Article",
    "NewsArticle",
    "BlogPosting",
    "Recipe",
    "Event",
    "JobPosting",
    "FAQPage",
    "QAPage",
    "ContactPage",
    "AboutPage",
    "CheckoutPage",
    "ProfilePage",
    "WebPage",
    "BreadcrumbList",
];

/// How a page came to be in its template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Assignment {
    /// Its structured data names the template, and its layout fits it.
    StructuredData,
    /// It has no structured data type, and its layout fits a typed template.
    LayoutWithoutMarkup,
    /// Its own type's template did not fit its layout, another one did.
    LayoutOverType,
    /// Clustered with pages of a similar layout, none of them typed.
    Shape,
}

impl Assignment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StructuredData => "structured_data",
            Self::LayoutWithoutMarkup => "layout_without_markup",
            Self::LayoutOverType => "layout_over_type",
            Self::Shape => "shape",
        }
    }

    pub fn from_stored(stored: &str) -> Option<Self> {
        match stored {
            "structured_data" => Some(Self::StructuredData),
            "layout_without_markup" => Some(Self::LayoutWithoutMarkup),
            "layout_over_type" => Some(Self::LayoutOverType),
            "shape" => Some(Self::Shape),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::StructuredData => "Structured data",
            Self::LayoutWithoutMarkup => "Layout, no markup",
            Self::LayoutOverType => "Layout, over its own type",
            Self::Shape => "Layout",
        }
    }
}

/// A page's shape with its paths interned for the run.
#[derive(Clone, Debug)]
struct Shape {
    weights: HashMap<u32, f64>,
    ancestors: HashMap<u32, Vec<u32>>,
}

#[derive(Default)]
struct PathIds {
    ids: HashMap<String, u32>,
}

impl PathIds {
    fn id(&mut self, path: String) -> u32 {
        let next = self.ids.len() as u32;
        *self.ids.entry(path).or_insert(next)
    }

    fn shape(&mut self, layout: &PageLayout, tables: &LayoutTables) -> Shape {
        let shape = page_shape(layout, tables);
        let mut weights = HashMap::with_capacity(shape.weights.len());
        for (path, weight) in shape.weights {
            weights.insert(self.id(path), weight);
        }
        let mut ancestors = HashMap::with_capacity(shape.ancestors.len());
        for (path, parents) in shape.ancestors {
            let id = self.id(path);
            let parent_ids = parents.into_iter().map(|parent| self.id(parent)).collect();
            ancestors.insert(id, parent_ids);
        }
        Shape { weights, ancestors }
    }
}

/// How often each path appears among some pages, for learning a template.
#[derive(Clone, Debug, Default)]
struct PathCounts {
    pages: usize,
    counts: HashMap<u32, (usize, f64)>,
}

impl PathCounts {
    fn add(&mut self, shape: &Shape, chrome: &HashSet<u32>) {
        self.pages += 1;
        for (path, weight) in &shape.weights {
            if chrome.contains(path) {
                continue;
            }
            let entry = self.counts.entry(*path).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += weight;
        }
    }

    /// The template, leaving out one of the counted pages when given, so a
    /// page is never tested against a template it helped learn.
    fn template(&self, leaving_out: Option<&Shape>, chrome: &HashSet<u32>) -> Template {
        let pages = self.pages - usize::from(leaving_out.is_some());
        let mut template = Template::default();
        if pages == 0 {
            return template;
        }
        for (path, (count, weight_sum)) in &self.counts {
            let (mut count, mut weight_sum) = (*count, *weight_sum);
            if let Some(own) = leaving_out.and_then(|shape| shape.weights.get(path))
                && !chrome.contains(path)
            {
                count -= 1;
                weight_sum -= own;
            }
            if count == 0 {
                continue;
            }
            let share = count as f64 / pages as f64;
            let weight = weight_sum / count as f64;
            if share >= CORE {
                template.core.insert(*path, weight);
            } else if share >= OPTIONAL && count >= 2 {
                template.optional.insert(*path, weight);
            }
        }
        template
    }
}

#[derive(Clone, Debug, Default)]
struct Template {
    core: HashMap<u32, f64>,
    optional: HashMap<u32, f64>,
}

#[derive(Clone, Copy, Debug)]
struct Membership {
    coverage: f64,
    passes: bool,
}

impl Template {
    fn contains(&self, path: &u32) -> bool {
        self.core.contains_key(path) || self.optional.contains_key(path)
    }

    fn membership(&self, shape: &Shape, chrome: &HashSet<u32>) -> Membership {
        let core_total: f64 = self.core.values().sum();
        let core_hit: f64 = self
            .core
            .iter()
            .filter(|(path, _)| shape.weights.contains_key(path))
            .map(|(_, weight)| weight)
            .sum();
        let mut page_total = 0.0;
        let mut explained = 0.0;
        for (path, weight) in &shape.weights {
            if chrome.contains(path) {
                continue;
            }
            page_total += weight;
            // Content inside a template region, such as manufacturer content
            // in a description, fills a slot of the template rather than
            // replacing it.
            let in_slot = shape.ancestors.get(path).is_some_and(|parents| {
                parents
                    .iter()
                    .any(|parent| !chrome.contains(parent) && self.contains(parent))
            });
            if self.contains(path) || in_slot {
                explained += weight;
            }
        }
        let coverage = if core_total == 0.0 {
            0.0
        } else {
            core_hit / core_total
        };
        let explained_share = if page_total == 0.0 {
            1.0
        } else {
            explained / page_total
        };
        Membership {
            coverage,
            passes: coverage >= COVERAGE && explained_share >= EXPLAINED,
        }
    }
}

fn weighted_jaccard(a: &Shape, b: &Shape, chrome: &HashSet<u32>) -> f64 {
    let mut shared = 0.0;
    let mut total = 0.0;
    for (path, weight_a) in &a.weights {
        if chrome.contains(path) {
            continue;
        }
        match b.weights.get(path) {
            Some(weight_b) => {
                shared += weight_a.min(*weight_b);
                total += weight_a.max(*weight_b);
            }
            None => total += weight_a,
        }
    }
    for (path, weight_b) in &b.weights {
        if !chrome.contains(path) && !a.weights.contains_key(path) {
            total += weight_b;
        }
    }
    if total == 0.0 { 1.0 } else { shared / total }
}

/// Clusters by average linkage until no two clusters are closer than `cut`.
fn average_linkage(shapes: &[Shape], chrome: &HashSet<u32>, cut: f64) -> Vec<Vec<usize>> {
    let count = shapes.len();
    let mut similarity = vec![vec![0.0; count]; count];
    for first in 0..count {
        for second in first + 1..count {
            let value = weighted_jaccard(&shapes[first], &shapes[second], chrome);
            similarity[first][second] = value;
            similarity[second][first] = value;
        }
    }
    let mut clusters: Vec<Vec<usize>> = (0..count).map(|index| vec![index]).collect();
    loop {
        let mut best: Option<(usize, usize, f64)> = None;
        for first in 0..clusters.len() {
            for second in first + 1..clusters.len() {
                let mut sum = 0.0;
                for a in &clusters[first] {
                    for b in &clusters[second] {
                        sum += similarity[*a][*b];
                    }
                }
                let average = sum / (clusters[first].len() * clusters[second].len()) as f64;
                if best.is_none_or(|(_, _, value)| average > value) {
                    best = Some((first, second, average));
                }
            }
        }
        match best {
            Some((first, second, average)) if average >= cut => {
                let merged = clusters.remove(second);
                clusters[first].extend(merged);
            }
            _ => return clusters,
        }
    }
}

/// A deterministic stand-in for random sampling: the same crawl always
/// learns from the same pages.
fn sample_rank(url: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in url.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Keeps the `SAMPLE_LIMIT` pages with the lowest rank.
#[derive(Default)]
struct Sample {
    members: Vec<(u64, String, Shape)>,
}

impl Sample {
    fn offer(&mut self, url: &str, shape: &Shape) {
        let rank = sample_rank(url);
        if self.members.len() < SAMPLE_LIMIT {
            self.members.push((rank, url.to_string(), shape.clone()));
            return;
        }
        if let Some((worst_index, worst)) = self
            .members
            .iter()
            .enumerate()
            .max_by_key(|(_, (rank, _, _))| *rank)
            .map(|(index, (rank, _, _))| (index, *rank))
            && rank < worst
        {
            self.members[worst_index] = (rank, url.to_string(), shape.clone());
        }
    }
}

/// A template found for the crawl, and how many pages joined it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateSummary {
    pub id: i64,
    pub name: String,
    /// The structured data type naming the template. `None` for templates
    /// found by layout alone.
    pub type_key: Option<String>,
    pub page_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Grouping {
    pub templates: Vec<TemplateSummary>,
    /// URL to template id and how the page joined it.
    pub pages: HashMap<String, (i64, Assignment)>,
}

struct TypedGroup {
    type_key: String,
    member_count: usize,
    counts: PathCounts,
    /// Sample members whose layout fits the others, by URL.
    sample_passes: HashMap<String, bool>,
    template: Template,
}

/// Groups the crawl's stored layouts into templates.
pub async fn group_crawl(pool: &SqlitePool, crawl_id: i64) -> Result<Grouping, sqlx::Error> {
    let tables = crate::storage::load_layout_tables(pool, crawl_id).await?;
    let mut path_ids = PathIds::default();

    // Pass 1: how common each path and type is, and samples per type.
    let mut page_count = 0usize;
    let mut path_frequency: HashMap<u32, usize> = HashMap::new();
    let mut type_frequency: HashMap<String, usize> = HashMap::new();
    let mut page_types: Vec<(String, Vec<String>)> = Vec::new();
    let mut samples: HashMap<String, Sample> = HashMap::new();
    for_each_layout(pool, crawl_id, |url, layout| {
        let shape = path_ids.shape(&layout, &tables);
        page_count += 1;
        for path in shape.weights.keys() {
            *path_frequency.entry(*path).or_default() += 1;
        }
        let types: Vec<String> = layout
            .types
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();
        for type_name in &types {
            *type_frequency.entry(type_name.clone()).or_default() += 1;
            if TYPE_PRIORITY.contains(&type_name.as_str()) {
                samples
                    .entry(type_name.clone())
                    .or_default()
                    .offer(&url, &shape);
            }
        }
        page_types.push((url, types));
    })
    .await?;
    if page_count == 0 {
        return Ok(Grouping::default());
    }

    let site_wide = |count: usize| count as f64 >= SITE_WIDE * page_count as f64;
    let chrome: HashSet<u32> = path_frequency
        .iter()
        .filter(|(_, count)| site_wide(**count))
        .map(|(path, _)| *path)
        .collect();
    let site_wide_types: HashSet<&String> = type_frequency
        .iter()
        .filter(|(_, count)| site_wide(**count))
        .map(|(type_name, _)| type_name)
        .collect();
    let type_key_of = |types: &[String]| -> Option<String> {
        TYPE_PRIORITY
            .iter()
            .find(|candidate| {
                types.iter().any(|type_name| {
                    type_name == *candidate && !site_wide_types.contains(type_name)
                })
            })
            .map(|type_name| (*type_name).to_string())
    };
    let type_keys: HashMap<String, Option<String>> = page_types
        .iter()
        .map(|(url, types)| (url.clone(), type_key_of(types)))
        .collect();

    // Templates per type, learned from the sample members that type names,
    // then again from only those whose layout fits the rest.
    let mut member_counts: HashMap<String, usize> = HashMap::new();
    for type_key in type_keys.values().flatten() {
        *member_counts.entry(type_key.clone()).or_default() += 1;
    }
    let mut typed_groups: Vec<TypedGroup> = Vec::new();
    for type_name in TYPE_PRIORITY {
        let Some(member_count) = member_counts.get(type_name).copied() else {
            continue;
        };
        let mut sample = samples.remove(type_name).unwrap_or_default();
        sample.members.retain(|(_, url, _)| {
            type_keys
                .get(url)
                .is_some_and(|key| key.as_deref() == Some(type_name))
        });
        let mut counts = PathCounts::default();
        for (_, _, shape) in &sample.members {
            counts.add(shape, &chrome);
        }
        let mut sample_passes = HashMap::new();
        for (_, url, shape) in &sample.members {
            let passes = member_count < 3
                || counts
                    .template(Some(shape), &chrome)
                    .membership(shape, &chrome)
                    .passes;
            sample_passes.insert(url.clone(), passes);
        }
        let mut fitting = PathCounts::default();
        for (_, url, shape) in &sample.members {
            if sample_passes.get(url).copied().unwrap_or(false) {
                fitting.add(shape, &chrome);
            }
        }
        let template = fitting.template(None, &chrome);
        typed_groups.push(TypedGroup {
            type_key: type_name.to_string(),
            member_count,
            counts,
            sample_passes,
            template,
        });
    }

    // Pass 2: every page against its own type's template, then the others.
    let mut assigned: HashMap<String, (usize, Assignment)> = HashMap::new();
    let mut leftovers: Vec<(String, Shape)> = Vec::new();
    let mut overflow: Vec<String> = Vec::new();
    for_each_layout(pool, crawl_id, |url, layout| {
        let shape = path_ids.shape(&layout, &tables);
        let type_key = type_keys.get(&url).cloned().flatten();
        let own_group = type_key
            .as_ref()
            .and_then(|key| typed_groups.iter().position(|group| &group.type_key == key));
        if let Some(group_index) = own_group {
            let group = &typed_groups[group_index];
            let passes = match group.sample_passes.get(&url) {
                Some(passes) => *passes,
                None if group.member_count < 3 => true,
                None => {
                    group
                        .counts
                        .template(None, &chrome)
                        .membership(&shape, &chrome)
                        .passes
                }
            };
            if passes {
                assigned.insert(url, (group_index, Assignment::StructuredData));
                return;
            }
        }
        let best = typed_groups
            .iter()
            .enumerate()
            .filter(|(index, _)| Some(*index) != own_group)
            .map(|(index, group)| (index, group.template.membership(&shape, &chrome)))
            .filter(|(_, membership)| membership.passes)
            .max_by(|a, b| a.1.coverage.total_cmp(&b.1.coverage));
        if let Some((group_index, _)) = best {
            let assignment = if type_key.is_some() {
                Assignment::LayoutOverType
            } else {
                Assignment::LayoutWithoutMarkup
            };
            assigned.insert(url, (group_index, assignment));
        } else if leftovers.len() < CLUSTER_LIMIT {
            leftovers.push((url, shape));
        } else {
            overflow.push(url);
        }
    })
    .await?;

    let mut grouping = Grouping::default();
    let mut typed_ids: HashMap<usize, i64> = HashMap::new();
    for (index, group) in typed_groups.iter().enumerate() {
        let page_count = assigned
            .values()
            .filter(|(group_index, _)| *group_index == index)
            .count();
        if page_count == 0 {
            continue;
        }
        let id = grouping.templates.len() as i64 + 1;
        typed_ids.insert(index, id);
        grouping.templates.push(TemplateSummary {
            id,
            name: group.type_key.clone(),
            type_key: Some(group.type_key.clone()),
            page_count,
        });
    }
    for (url, (group_index, assignment)) in assigned {
        if let Some(id) = typed_ids.get(&group_index) {
            grouping.pages.insert(url, (*id, assignment));
        }
    }

    let leftover_shapes: Vec<Shape> = leftovers.iter().map(|(_, shape)| shape.clone()).collect();
    let mut clusters = average_linkage(&leftover_shapes, &chrome, SHAPE_CUT);
    // Biggest first, so the names read in order of size.
    clusters.sort_by_key(|cluster| std::cmp::Reverse(cluster.len()));
    let mut shape_templates: Vec<(i64, Template)> = Vec::new();
    for (number, cluster) in clusters.iter().enumerate() {
        let id = grouping.templates.len() as i64 + 1;
        let mut counts = PathCounts::default();
        for member in cluster {
            counts.add(&leftover_shapes[*member], &chrome);
            grouping
                .pages
                .insert(leftovers[*member].0.clone(), (id, Assignment::Shape));
        }
        shape_templates.push((id, counts.template(None, &chrome)));
        grouping.templates.push(TemplateSummary {
            id,
            name: format!("Layout {}", number + 1),
            type_key: None,
            page_count: cluster.len(),
        });
    }

    // Pass 3, only for crawls with more leftovers than the clustering takes.
    if !overflow.is_empty() {
        let overflow: HashSet<String> = overflow.into_iter().collect();
        let mut joined: Vec<(String, Option<i64>)> = Vec::new();
        for_each_layout(pool, crawl_id, |url, layout| {
            if !overflow.contains(&url) {
                return;
            }
            let shape = path_ids.shape(&layout, &tables);
            let best = shape_templates
                .iter()
                .map(|(id, template)| (*id, template.membership(&shape, &chrome)))
                .filter(|(_, membership)| membership.passes)
                .max_by(|a, b| a.1.coverage.total_cmp(&b.1.coverage))
                .map(|(id, _)| id);
            joined.push((url, best));
        })
        .await?;
        let unmatched = joined.iter().filter(|(_, id)| id.is_none()).count();
        let other_id = (unmatched > 0).then(|| {
            let id = grouping.templates.len() as i64 + 1;
            grouping.templates.push(TemplateSummary {
                id,
                name: "Other layouts".to_string(),
                type_key: None,
                page_count: unmatched,
            });
            id
        });
        for (url, id) in joined {
            let Some(id) = id.or(other_id) else {
                continue;
            };
            if let Some(template) = grouping
                .templates
                .iter_mut()
                .find(|template| template.id == id && template.name != "Other layouts")
            {
                template.page_count += 1;
            }
            grouping.pages.insert(url, (id, Assignment::Shape));
        }
    }

    Ok(grouping)
}

/// Groups the crawl's layouts and stores the result. A crawl without
/// captured layouts is left alone.
pub async fn run_template_grouping(pool: &SqlitePool, crawl_id: i64) -> Result<(), sqlx::Error> {
    if crate::storage::count_page_layouts(pool, crawl_id).await? == 0 {
        return Ok(());
    }
    let grouping = group_crawl(pool, crawl_id).await?;
    crate::storage::save_layout_grouping(pool, crawl_id, &grouping).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(paths: &[(u32, f64)]) -> Shape {
        Shape {
            weights: paths.iter().copied().collect(),
            ancestors: HashMap::new(),
        }
    }

    #[test]
    fn leaving_a_page_out_of_its_template() {
        let chrome = HashSet::new();
        let first = shape(&[(1, 10.0), (2, 10.0), (3, 5.0)]);
        let second = shape(&[(1, 10.0), (2, 10.0)]);
        let third = shape(&[(1, 10.0), (2, 10.0)]);
        let mut counts = PathCounts::default();
        for page in [&first, &second, &third] {
            counts.add(page, &chrome);
        }
        let with_all = counts.template(None, &chrome);
        assert!(
            !with_all.contains(&3),
            "a path on one page of three is neither core nor optional"
        );
        let without_first = counts.template(Some(&first), &chrome);
        assert!(!without_first.contains(&3), "only the left out page had it");
        assert!(without_first.core.contains_key(&1));
        assert!(without_first.membership(&second, &chrome).passes);
        assert!(
            !without_first
                .membership(&shape(&[(9, 50.0)]), &chrome)
                .passes,
            "a page sharing nothing fails"
        );
    }

    #[test]
    fn content_inside_a_template_region_is_explained() {
        let chrome = HashSet::new();
        let template = Template {
            core: [(1, 10.0), (2, 10.0)].into_iter().collect(),
            optional: HashMap::new(),
        };
        let mut page = shape(&[(1, 10.0), (2, 10.0), (7, 30.0)]);
        assert!(!template.membership(&page, &chrome).passes);
        page.ancestors.insert(7, vec![2]);
        assert!(template.membership(&page, &chrome).passes);
    }

    #[test]
    fn similar_layouts_cluster_and_different_ones_do_not() {
        let chrome = HashSet::new();
        let shapes = vec![
            shape(&[(1, 10.0), (2, 10.0)]),
            shape(&[(1, 10.0), (2, 9.0)]),
            shape(&[(5, 10.0), (6, 10.0)]),
        ];
        let mut clusters = average_linkage(&shapes, &chrome, SHAPE_CUT);
        clusters.sort();
        assert_eq!(clusters, [vec![0, 1], vec![2]]);
    }

    #[test]
    fn samples_keep_the_lowest_ranks() {
        let mut sample = Sample::default();
        let page = shape(&[(1, 1.0)]);
        for index in 0..SAMPLE_LIMIT + 50 {
            sample.offer(&format!("https://example.com/{index}"), &page);
        }
        assert_eq!(sample.members.len(), SAMPLE_LIMIT);
        let worst_kept = sample.members.iter().map(|(rank, _, _)| *rank).max();
        let best_dropped = (0..SAMPLE_LIMIT + 50)
            .map(|index| sample_rank(&format!("https://example.com/{index}")))
            .filter(|rank| !sample.members.iter().any(|(kept, _, _)| kept == rank))
            .min();
        assert!(worst_kept < best_dropped);
    }

    #[test]
    fn assignments_round_trip_through_storage() {
        for assignment in [
            Assignment::StructuredData,
            Assignment::LayoutWithoutMarkup,
            Assignment::LayoutOverType,
            Assignment::Shape,
        ] {
            assert_eq!(
                Assignment::from_stored(assignment.as_str()),
                Some(assignment)
            );
        }
        assert_eq!(Assignment::from_stored("unknown"), None);
    }
}
