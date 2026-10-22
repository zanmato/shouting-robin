//! Page layout: the boxes a rendered page draws, captured in Chrome by
//! `assets/js/layout_capture.js`, stored compactly per page and compared
//! between two crawls of the same URL.
//!
//! A box is identified by a normalized key (its tag and first stable class)
//! and its position among its siblings. Siblings that share a key form a
//! list, whose items are compared as a whole: a product swapped for another
//! is content, not a layout change.

use std::collections::{HashMap, HashSet};
use std::io::{Read as _, Write as _};

use anyhow::{Context as _, bail};

pub const LAYOUT_CAPTURE_JS: &str = include_str!("../../assets/js/layout_capture.js");

/// The id of the script element the capture script stashes its result in.
pub const LAYOUT_STASH_ID: &str = "__sr_layout";

/// The capture the page stashed in its HTML, and the HTML without it. The
/// stash is the crawler's own and must not count towards the page's size or
/// content, so it is taken out before anything reads the HTML.
pub fn take_stashed_capture(html: &[u8]) -> Option<(Vec<u8>, String)> {
    let marker = format!("id=\"{LAYOUT_STASH_ID}\"");
    let marker_at = find_bytes(html, marker.as_bytes(), 0)?;
    let start = html[..marker_at]
        .windows(b"<script".len())
        .rposition(|window| window.eq_ignore_ascii_case(b"<script"))?;
    let content_start = marker_at + find_bytes(&html[marker_at..], b">", 0)? + 1;
    let content_end = find_bytes(html, b"</script>", content_start)?;
    let end = content_end + b"</script>".len();
    let json = String::from_utf8(html[content_start..content_end].to_vec()).ok()?;
    let mut cleaned = Vec::with_capacity(html.len() - (end - start));
    cleaned.extend_from_slice(&html[..start]);
    cleaned.extend_from_slice(&html[end..]);
    Some((cleaned, json))
}

fn find_bytes(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack
        .get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}

/// The computed style values recorded per box, in the order the capture
/// script sends them.
pub const STYLE_PROPS: [&str; 20] = [
    "color",
    "backgroundColor",
    "backgroundImage",
    "fontFamily",
    "fontSize",
    "fontWeight",
    "lineHeight",
    "textAlign",
    "textTransform",
    "textDecorationLine",
    "borderTopWidth",
    "borderTopColor",
    "borderRadius",
    "paddingTop",
    "paddingBottom",
    "paddingLeft",
    "paddingRight",
    "display",
    "flexDirection",
    "opacity",
];

/// Siblings sharing a key that count as a list.
const LIST_MIN: usize = 3;
/// Pixels a box may shift or grow before it counts as a change.
const TOLERANCE: i32 = 4;
/// Trailing keys that make a grouping path. A modifier class on an ancestor
/// would otherwise change the path of its whole subtree.
pub const PATH_WINDOW: usize = 4;

const FLAG_FIXED: u8 = 1;
const FLAG_IMAGE: u8 = 2;
const FLAG_CLIP: u8 = 4;
const ENCODING_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn area(&self) -> f64 {
        f64::from(self.width.max(0)) * f64::from(self.height.max(0))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBox {
    pub parent: Option<usize>,
    /// Index into the crawl's key table.
    pub key: u32,
    /// In page coordinates, or viewport coordinates for a fixed box.
    pub rect: Rect,
    /// Index into the crawl's style table.
    pub style: u32,
    /// Hash of the text the box holds directly, 0 for none.
    pub text_hash: u32,
    pub image_hash: Option<u32>,
    pub fixed: bool,
    /// The part an overflow ancestor leaves visible, when that is less than
    /// the whole box.
    pub clip: Option<Rect>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageLayout {
    pub width: u32,
    pub height: u32,
    /// Structured data types describing the page itself.
    pub types: Vec<String>,
    pub boxes: Vec<LayoutBox>,
}

/// The keys and styles of one crawl. Pages store indices into them, since
/// most pages of a site share nearly all of both.
#[derive(Clone, Debug, Default)]
pub struct LayoutTables {
    keys: Vec<String>,
    styles: Vec<Vec<String>>,
    key_ids: HashMap<String, u32>,
    style_ids: HashMap<Vec<String>, u32>,
}

/// Table entries a capture added, which the caller stores.
#[derive(Debug, Default)]
pub struct NewTableEntries {
    pub keys: Vec<(u32, String)>,
    pub styles: Vec<(u32, Vec<String>)>,
}

impl LayoutTables {
    pub fn from_rows(keys: Vec<(u32, String)>, styles: Vec<(u32, Vec<String>)>) -> Self {
        let mut tables = Self::default();
        let mut keys = keys;
        keys.sort_by_key(|(id, _)| *id);
        for (id, key) in keys {
            if id as usize == tables.keys.len() {
                tables.key_ids.insert(key.clone(), id);
                tables.keys.push(key);
            }
        }
        let mut styles = styles;
        styles.sort_by_key(|(id, _)| *id);
        for (id, style) in styles {
            if id as usize == tables.styles.len() {
                tables.style_ids.insert(style.clone(), id);
                tables.styles.push(style);
            }
        }
        tables
    }

    pub fn key(&self, id: u32) -> &str {
        self.keys.get(id as usize).map_or("", String::as_str)
    }

    pub fn style(&self, id: u32) -> &[String] {
        self.styles.get(id as usize).map_or(&[], Vec::as_slice)
    }

    pub fn style_value(&self, id: u32, prop: &str) -> &str {
        let index = STYLE_PROPS.iter().position(|name| *name == prop);
        index
            .and_then(|index| self.style(id).get(index))
            .map_or("", String::as_str)
    }

    fn intern_key(&mut self, key: String, added: &mut NewTableEntries) -> u32 {
        if let Some(id) = self.key_ids.get(&key) {
            return *id;
        }
        let id = self.keys.len() as u32;
        self.key_ids.insert(key.clone(), id);
        self.keys.push(key.clone());
        added.keys.push((id, key));
        id
    }

    fn intern_style(&mut self, style: Vec<String>, added: &mut NewTableEntries) -> u32 {
        if let Some(id) = self.style_ids.get(&style) {
            return *id;
        }
        let id = self.styles.len() as u32;
        self.style_ids.insert(style.clone(), id);
        self.styles.push(style.clone());
        added.styles.push((id, style));
        id
    }
}

/// What the capture script returns.
#[derive(serde::Deserialize)]
pub struct RawCapture {
    props: Vec<String>,
    types: Vec<String>,
    width: u32,
    height: u32,
    styles: Vec<Vec<String>>,
    nodes: Vec<RawNode>,
}

/// One box as the capture script sends it: parent index, tag, class
/// attribute, test id, x, y, width, height, style index, text hash, image
/// hash, flags and the visible clip.
#[derive(serde::Deserialize)]
struct RawNode(
    i64,
    String,
    String,
    Option<String>,
    i32,
    i32,
    i32,
    i32,
    u32,
    u32,
    Option<u32>,
    u8,
    Option<[i32; 4]>,
);

impl PageLayout {
    /// Turns a capture into a layout, adding its keys and styles to the
    /// crawl's tables.
    pub fn from_capture(
        capture: RawCapture,
        tables: &mut LayoutTables,
    ) -> anyhow::Result<(Self, NewTableEntries)> {
        if capture.props != STYLE_PROPS {
            bail!("the capture script records other style properties than the reader expects");
        }
        let mut added = NewTableEntries::default();
        let mut style_ids = Vec::with_capacity(capture.styles.len());
        for style in capture.styles {
            if style.len() != STYLE_PROPS.len() {
                bail!("a captured style has {} values", style.len());
            }
            style_ids.push(tables.intern_style(style, &mut added));
        }
        let mut boxes = Vec::with_capacity(capture.nodes.len());
        for (index, node) in capture.nodes.into_iter().enumerate() {
            let RawNode(
                parent,
                tag,
                class,
                testid,
                x,
                y,
                width,
                height,
                style,
                text_hash,
                image_hash,
                flags,
                clip,
            ) = node;
            let parent = match usize::try_from(parent) {
                Ok(parent) if parent < index => Some(parent),
                Ok(_) => bail!("box {index} names a parent that comes after it"),
                Err(_) => None,
            };
            let style = *style_ids
                .get(style as usize)
                .with_context(|| format!("box {index} names a style that was not captured"))?;
            let key = normalize_key(&tag, &class, testid.as_deref());
            boxes.push(LayoutBox {
                parent,
                key: tables.intern_key(key, &mut added),
                rect: Rect {
                    x,
                    y,
                    width,
                    height,
                },
                style,
                text_hash,
                image_hash,
                fixed: flags & FLAG_FIXED != 0,
                clip: clip.map(|[x, y, width, height]| Rect {
                    x,
                    y,
                    width,
                    height,
                }),
            });
        }
        Ok((
            Self {
                width: capture.width,
                height: capture.height,
                types: capture.types,
                boxes,
            },
            added,
        ))
    }

    pub fn children(&self) -> Vec<Vec<usize>> {
        let mut children = vec![Vec::new(); self.boxes.len()];
        for (index, layout_box) in self.boxes.iter().enumerate() {
            if let Some(parent) = layout_box.parent
                && let Some(siblings) = children.get_mut(parent)
            {
                siblings.push(index);
            }
        }
        children
    }

    /// Stores the layout as varint columns, deflated. Positions are relative
    /// to the parent, which makes the repeated structure of a page compress.
    pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(self.boxes.len() * 16);
        bytes.push(ENCODING_VERSION);
        write_unsigned(&mut bytes, u64::from(self.width));
        write_unsigned(&mut bytes, u64::from(self.height));
        write_unsigned(&mut bytes, self.types.len() as u64);
        for type_name in &self.types {
            write_unsigned(&mut bytes, type_name.len() as u64);
            bytes.extend_from_slice(type_name.as_bytes());
        }
        write_unsigned(&mut bytes, self.boxes.len() as u64);
        for (index, layout_box) in self.boxes.iter().enumerate() {
            let distance = layout_box.parent.map_or(0, |parent| index - parent);
            write_unsigned(&mut bytes, distance as u64);
        }
        for layout_box in &self.boxes {
            write_unsigned(&mut bytes, u64::from(layout_box.key));
        }
        for (index, layout_box) in self.boxes.iter().enumerate() {
            let origin = self.parent_origin(index);
            write_signed(&mut bytes, i64::from(layout_box.rect.x - origin.0));
            write_signed(&mut bytes, i64::from(layout_box.rect.y - origin.1));
        }
        for layout_box in &self.boxes {
            write_signed(&mut bytes, i64::from(layout_box.rect.width));
            write_signed(&mut bytes, i64::from(layout_box.rect.height));
        }
        for layout_box in &self.boxes {
            write_unsigned(&mut bytes, u64::from(layout_box.style));
        }
        for layout_box in &self.boxes {
            bytes.extend_from_slice(&layout_box.text_hash.to_le_bytes());
        }
        for layout_box in &self.boxes {
            let mut flags = 0;
            if layout_box.fixed {
                flags |= FLAG_FIXED;
            }
            if layout_box.image_hash.is_some() {
                flags |= FLAG_IMAGE;
            }
            if layout_box.clip.is_some() {
                flags |= FLAG_CLIP;
            }
            bytes.push(flags);
        }
        for layout_box in &self.boxes {
            if let Some(image_hash) = layout_box.image_hash {
                bytes.extend_from_slice(&image_hash.to_le_bytes());
            }
        }
        for layout_box in &self.boxes {
            if let Some(clip) = layout_box.clip {
                write_signed(&mut bytes, i64::from(clip.x - layout_box.rect.x));
                write_signed(&mut bytes, i64::from(clip.y - layout_box.rect.y));
                write_signed(&mut bytes, i64::from(clip.width));
                write_signed(&mut bytes, i64::from(clip.height));
            }
        }
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&bytes)?;
        Ok(encoder.finish()?)
    }

    pub fn decode(compressed: &[u8]) -> anyhow::Result<Self> {
        let mut bytes = Vec::new();
        flate2::read::DeflateDecoder::new(compressed)
            .read_to_end(&mut bytes)
            .context("stored layout is not deflated")?;
        let mut reader = ByteReader {
            bytes: &bytes,
            position: 0,
        };
        let version = reader.byte()?;
        if version != ENCODING_VERSION {
            bail!("stored layout has unknown version {version}");
        }
        let width = u32::try_from(reader.unsigned()?)?;
        let height = u32::try_from(reader.unsigned()?)?;
        let type_count = reader.length()?;
        let mut types = Vec::with_capacity(type_count);
        for _ in 0..type_count {
            let length = reader.length()?;
            types.push(String::from_utf8(reader.take(length)?.to_vec())?);
        }
        let count = reader.length()?;
        let mut boxes: Vec<LayoutBox> = Vec::with_capacity(count);
        for index in 0..count {
            let distance = reader.length()?;
            let parent = match distance {
                0 => None,
                distance if distance <= index => Some(index - distance),
                _ => bail!("stored box {index} has a parent before the first box"),
            };
            boxes.push(LayoutBox {
                parent,
                key: 0,
                rect: Rect::default(),
                style: 0,
                text_hash: 0,
                image_hash: None,
                fixed: false,
                clip: None,
            });
        }
        for layout_box in &mut boxes {
            layout_box.key = u32::try_from(reader.unsigned()?)?;
        }
        let mut layout = Self {
            width,
            height,
            types,
            boxes,
        };
        for index in 0..count {
            let origin = layout.parent_origin(index);
            let x = origin.0 + i32::try_from(reader.signed()?)?;
            let y = origin.1 + i32::try_from(reader.signed()?)?;
            if let Some(layout_box) = layout.boxes.get_mut(index) {
                layout_box.rect.x = x;
                layout_box.rect.y = y;
            }
        }
        for layout_box in &mut layout.boxes {
            layout_box.rect.width = i32::try_from(reader.signed()?)?;
            layout_box.rect.height = i32::try_from(reader.signed()?)?;
        }
        for layout_box in &mut layout.boxes {
            layout_box.style = u32::try_from(reader.unsigned()?)?;
        }
        for layout_box in &mut layout.boxes {
            layout_box.text_hash = reader.u32_le()?;
        }
        let mut flags = Vec::with_capacity(count);
        for layout_box in &mut layout.boxes {
            let flag = reader.byte()?;
            layout_box.fixed = flag & FLAG_FIXED != 0;
            flags.push(flag);
        }
        for (layout_box, flag) in layout.boxes.iter_mut().zip(&flags) {
            if flag & FLAG_IMAGE != 0 {
                layout_box.image_hash = Some(reader.u32_le()?);
            }
        }
        for (layout_box, flag) in layout.boxes.iter_mut().zip(&flags) {
            if flag & FLAG_CLIP != 0 {
                let x = layout_box.rect.x + i32::try_from(reader.signed()?)?;
                let y = layout_box.rect.y + i32::try_from(reader.signed()?)?;
                layout_box.clip = Some(Rect {
                    x,
                    y,
                    width: i32::try_from(reader.signed()?)?,
                    height: i32::try_from(reader.signed()?)?,
                });
            }
        }
        Ok(layout)
    }

    /// Where positions of box `index` are measured from in the encoding.
    fn parent_origin(&self, index: usize) -> (i32, i32) {
        self.boxes
            .get(index)
            .and_then(|layout_box| layout_box.parent)
            .and_then(|parent| self.boxes.get(parent))
            .map_or((0, 0), |parent| (parent.rect.x, parent.rect.y))
    }
}

fn write_unsigned(bytes: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        bytes.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

fn write_signed(bytes: &mut Vec<u8>, value: i64) {
    write_unsigned(bytes, ((value << 1) ^ (value >> 63)) as u64);
}

struct ByteReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl ByteReader<'_> {
    fn byte(&mut self) -> anyhow::Result<u8> {
        let byte = *self
            .bytes
            .get(self.position)
            .context("stored layout ends early")?;
        self.position += 1;
        Ok(byte)
    }

    fn take(&mut self, length: usize) -> anyhow::Result<&[u8]> {
        let end = self
            .position
            .checked_add(length)
            .context("stored layout length overflows")?;
        let slice = self
            .bytes
            .get(self.position..end)
            .context("stored layout ends early")?;
        self.position = end;
        Ok(slice)
    }

    fn unsigned(&mut self) -> anyhow::Result<u64> {
        let mut value = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        bail!("stored layout has an overlong number")
    }

    fn length(&mut self) -> anyhow::Result<usize> {
        let value = self.unsigned()?;
        let length = usize::try_from(value)?;
        if length > self.bytes.len() * 8 + 64 {
            bail!("stored layout has an impossible length {length}");
        }
        Ok(length)
    }

    fn signed(&mut self) -> anyhow::Result<i64> {
        let value = self.unsigned()?;
        Ok((value >> 1) as i64 ^ -((value & 1) as i64))
    }

    fn u32_le(&mut self) -> anyhow::Result<u32> {
        let slice = self.take(4)?;
        let array: [u8; 4] = slice.try_into()?;
        Ok(u32::from_le_bytes(array))
    }
}

/// A box's identity across pages and crawls: the tag and its first class
/// that is neither generated nor a state, or its test id.
pub fn normalize_key(tag: &str, class: &str, testid: Option<&str>) -> String {
    let tag = tag.to_ascii_lowercase();
    let testid = testid
        .filter(|testid| !testid.is_empty())
        .map(replace_digit_runs);
    let stable_class = class
        .split_ascii_whitespace()
        .find(|name| !is_generated_class(name) && !is_state_class(name));
    let mut key = tag;
    if let Some(testid) = testid {
        key.push('[');
        key.push_str(&testid);
        key.push(']');
    }
    if let Some(name) = stable_class {
        key.push('.');
        key.push_str(name);
    }
    key
}

fn replace_digit_runs(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_digits = false;
    for character in text.chars() {
        if character.is_ascii_digit() {
            if !in_digits {
                out.push('#');
            }
            in_digits = true;
        } else {
            out.push(character);
            in_digits = false;
        }
    }
    out
}

/// Class names a build tool generated, which change between deploys.
fn is_generated_class(name: &str) -> bool {
    let bytes = name.as_bytes();
    let mut digit_run = 0;
    for byte in bytes {
        if byte.is_ascii_digit() {
            digit_run += 1;
            if digit_run >= 3 {
                return true;
            }
        } else {
            digit_run = 0;
        }
    }
    let lower = name.to_ascii_lowercase();
    for prefix in ["css-", "sc-", "jsx-", "emotion-", "svelte-", "astro-"] {
        if let Some(rest) = lower.strip_prefix(prefix)
            && rest.len() >= 4
            && rest
                .chars()
                .take(4)
                .all(|character| character.is_ascii_alphanumeric())
        {
            return true;
        }
    }
    let has_digit = bytes.iter().any(u8::is_ascii_digit);
    // CSS modules, such as Price_root__a8F3k.
    if has_digit {
        let tail_charset =
            |byte: &u8| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'-';
        for index in 0..bytes.len().saturating_sub(1) {
            let pair = &bytes[index..index + 2];
            if (pair == b"__" || pair == b"--")
                && bytes.len() - (index + 2) >= 5
                && bytes[index + 2..].iter().all(tail_charset)
            {
                return true;
            }
        }
    }
    // A bare random token.
    bytes.len() >= 5
        && bytes.iter().all(u8::is_ascii_alphanumeric)
        && bytes.iter().any(u8::is_ascii_lowercase)
        && bytes.iter().any(u8::is_ascii_uppercase)
        && has_digit
}

const STATE_WORDS: [&str; 21] = [
    "active",
    "selected",
    "open",
    "opened",
    "current",
    "loaded",
    "lazyloaded",
    "loading",
    "visible",
    "hidden",
    "show",
    "shown",
    "focus",
    "focused",
    "hover",
    "disabled",
    "expanded",
    "collapsed",
    "sticky",
    "scrolled",
    "in-view",
];

/// Class names that describe a moment, such as `is-open`, rather than what
/// the element is.
fn is_state_class(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if ["is-", "has-", "js-"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
    {
        return true;
    }
    STATE_WORDS.iter().any(|word| {
        lower == *word
            || lower.strip_suffix(word).is_some_and(|rest| {
                rest.ends_with('-') || rest.ends_with('_') || rest.ends_with(':')
            })
    })
}

/// The tag a key starts with.
pub fn key_tag(key: &str) -> &str {
    key.split(['.', '[']).next().unwrap_or(key)
}

const PAINTING_TAGS: [&str; 9] = [
    "img", "svg", "video", "canvas", "iframe", "input", "select", "textarea", "picture",
];

/// Whether the box itself draws anything. Wrappers that only position their
/// children paint nothing, and drawing them is most of a wireframe's clutter.
pub fn paints(layout_box: &LayoutBox, tables: &LayoutTables) -> bool {
    if layout_box.text_hash != 0 {
        return true;
    }
    if PAINTING_TAGS.contains(&key_tag(tables.key(layout_box.key))) {
        return true;
    }
    let background = tables.style_value(layout_box.style, "backgroundColor");
    if !background.is_empty() && background != "rgba(0, 0, 0, 0)" && background != "transparent" {
        return true;
    }
    let image = tables.style_value(layout_box.style, "backgroundImage");
    if !image.is_empty() && image != "none" {
        return true;
    }
    let border = tables.style_value(layout_box.style, "borderTopWidth");
    !border.is_empty() && border != "0px"
}

/// Paths that tell boxes apart within one page: each box's chain of keys
/// with its position among same-keyed siblings, or `*` for list items.
struct IndexedPaths {
    paths: Vec<String>,
    lists: HashSet<String>,
}

fn indexed_paths(
    layout: &PageLayout,
    tables: &LayoutTables,
    forced_lists: &HashSet<String>,
) -> IndexedPaths {
    let children = layout.children();
    let mut paths = vec![String::new(); layout.boxes.len()];
    let mut lists = HashSet::new();
    let mut stack: Vec<usize> = Vec::new();
    for (index, layout_box) in layout.boxes.iter().enumerate() {
        if layout_box.parent.is_none() {
            paths[index] = format!("{}#0", tables.key(layout_box.key));
            stack.push(index);
        }
    }
    // Roots are visited in reverse so the walk keeps document order, which
    // no caller relies on but makes the paths easy to read when debugging.
    stack.reverse();
    while let Some(index) = stack.pop() {
        let Some(kids) = children.get(index) else {
            continue;
        };
        let mut totals: HashMap<u32, usize> = HashMap::new();
        for child in kids {
            if let Some(child_box) = layout.boxes.get(*child) {
                *totals.entry(child_box.key).or_default() += 1;
            }
        }
        let mut seen: HashMap<u32, usize> = HashMap::new();
        let parent_path = paths[index].clone();
        for child in kids {
            let Some(child_box) = layout.boxes.get(*child) else {
                continue;
            };
            let key = tables.key(child_box.key);
            let position = seen.entry(child_box.key).or_default();
            let list_key = format!("{parent_path}|{key}");
            let is_list = totals.get(&child_box.key).copied().unwrap_or(0) >= LIST_MIN
                || forced_lists.contains(&list_key);
            let segment = if is_list {
                "*".to_string()
            } else {
                position.to_string()
            };
            *position += 1;
            if is_list {
                lists.insert(list_key);
            }
            paths[*child] = format!("{parent_path}>{key}#{segment}");
        }
        for child in kids.iter().rev() {
            stack.push(*child);
        }
    }
    IndexedPaths { paths, lists }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayoutChangeKind {
    Added,
    Removed,
    Moved,
    Resized,
    Restyled,
    /// A list gained or lost items.
    Count,
    /// The text a box holds changed. Content, not layout.
    Text,
    /// An image's source changed. Content, not layout.
    Image,
}

impl LayoutChangeKind {
    pub fn is_layout(self) -> bool {
        !matches!(self, Self::Text | Self::Image)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::Moved => "moved",
            Self::Resized => "resized",
            Self::Restyled => "restyled",
            Self::Count => "item count",
            Self::Text => "text",
            Self::Image => "image",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayoutChange {
    pub path: String,
    pub kinds: Vec<LayoutChangeKind>,
    pub details: Vec<String>,
    pub list: bool,
    /// The boxes at this path before and after, as indices into each layout.
    pub before_boxes: Vec<usize>,
    pub after_boxes: Vec<usize>,
}

impl LayoutChange {
    /// The key of the box that changed, such as `h1.product-title`.
    pub fn label(&self) -> &str {
        let last = self.path.rsplit('>').next().unwrap_or(&self.path);
        last.rsplit_once('#').map_or(last, |(key, _)| key)
    }

    pub fn is_layout(&self) -> bool {
        self.kinds.iter().any(|kind| kind.is_layout())
    }

    pub fn kinds_text(&self) -> String {
        self.kinds
            .iter()
            .map(|kind| kind.label())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The changes between two captures of one URL that are not explained by
/// another change: a box that only grew because something inside it changed,
/// or only moved because something before it did, is left out.
#[derive(Clone, Debug, Default)]
pub struct LayoutDiff {
    pub changes: Vec<LayoutChange>,
    /// Changes left out as consequences of the ones listed.
    pub consequences: usize,
}

impl LayoutDiff {
    pub fn layout_changes(&self) -> impl Iterator<Item = &LayoutChange> {
        self.changes.iter().filter(|change| change.is_layout())
    }

    pub fn layout_change_count(&self) -> usize {
        self.layout_changes().count()
    }

    pub fn content_change_count(&self) -> usize {
        self.changes
            .iter()
            .filter(|change| !change.is_layout())
            .count()
    }

    /// A short line naming the layout changes, for the Changes tab.
    pub fn summary(&self) -> String {
        let parts: Vec<String> = self
            .layout_changes()
            .map(|change| {
                let kinds: Vec<&str> = change
                    .kinds
                    .iter()
                    .filter(|kind| kind.is_layout())
                    .map(|kind| kind.label())
                    .collect();
                format!("{} {}", change.label(), kinds.join(", "))
            })
            .collect();
        match parts.len() {
            0 => String::new(),
            1..=3 => parts.join("; "),
            count => format!("{}; and {} more", parts[..3].join("; "), count - 3),
        }
    }
}

/// Compares two captures of the same URL. Each layout is read with its own
/// crawl's tables.
pub fn diff_layouts(
    before: &PageLayout,
    before_tables: &LayoutTables,
    after: &PageLayout,
    after_tables: &LayoutTables,
) -> LayoutDiff {
    // A list that drops below LIST_MIN items on one side is still a list, or
    // its items would show up as removed and added.
    let first_before = indexed_paths(before, before_tables, &HashSet::new());
    let first_after = indexed_paths(after, after_tables, &HashSet::new());
    let lists: HashSet<String> = first_before
        .lists
        .union(&first_after.lists)
        .cloned()
        .collect();
    let paths_before = indexed_paths(before, before_tables, &lists).paths;
    let paths_after = indexed_paths(after, after_tables, &lists).paths;

    let bucket = |paths: &[String]| {
        let mut map: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, path) in paths.iter().enumerate() {
            map.entry(path.clone()).or_default().push(index);
        }
        map
    };
    let before_by_path = bucket(&paths_before);
    let after_by_path = bucket(&paths_after);
    let sides = Sides {
        before,
        before_tables,
        after,
        after_tables,
        before_by_path: &before_by_path,
        after_by_path: &after_by_path,
    };

    let all_paths: HashSet<&String> = before_by_path.keys().chain(after_by_path.keys()).collect();
    let empty = Vec::new();
    let mut changes: HashMap<String, LayoutChange> = HashMap::new();
    for path in all_paths {
        let boxes_before = before_by_path.get(path).unwrap_or(&empty);
        let boxes_after = after_by_path.get(path).unwrap_or(&empty);
        let list = path.contains("#*");
        let found = if list {
            sides.compare_list(path, boxes_before, boxes_after)
        } else {
            sides.compare_single(boxes_before.first().copied(), boxes_after.first().copied())
        };
        if let Some((kinds, details)) = found {
            changes.insert(
                path.clone(),
                LayoutChange {
                    path: path.clone(),
                    kinds,
                    details,
                    list,
                    before_boxes: boxes_before.clone(),
                    after_boxes: boxes_after.clone(),
                },
            );
        }
    }

    let parent_path = |path: &str| path.rfind('>').map(|end| path[..end].to_string());
    let mut has_changed_descendant: HashSet<String> = HashSet::new();
    for path in changes.keys() {
        let mut parent = parent_path(path);
        while let Some(current) = parent {
            parent = parent_path(&current);
            has_changed_descendant.insert(current);
        }
    }

    let children_before = before.children();
    let mut roots = Vec::new();
    let mut consequences = 0;
    for change in changes.values() {
        let parent_change = parent_path(&change.path).and_then(|parent| changes.get(&parent));
        // Inside an added or removed box: part of that change.
        if parent_change.is_some_and(|parent| {
            parent.kinds.contains(&LayoutChangeKind::Added)
                || parent.kinds.contains(&LayoutChangeKind::Removed)
        }) {
            consequences += 1;
            continue;
        }
        let mut kinds = change.kinds.clone();
        if kinds.contains(&LayoutChangeKind::Resized)
            && has_changed_descendant.contains(&change.path)
        {
            kinds.retain(|kind| *kind != LayoutChangeKind::Resized);
        }
        if kinds.contains(&LayoutChangeKind::Moved) {
            let earlier_changed = change.before_boxes.first().is_some_and(|node| {
                let siblings = before.boxes[*node]
                    .parent
                    .and_then(|parent| children_before.get(parent))
                    .map_or(&[][..], Vec::as_slice);
                siblings
                    .iter()
                    .take_while(|sibling| *sibling != node)
                    .any(|sibling| changes.contains_key(&paths_before[*sibling]))
            });
            if parent_change.is_some() || earlier_changed {
                kinds.retain(|kind| *kind != LayoutChangeKind::Moved);
            }
        }
        if kinds.is_empty() {
            consequences += 1;
            continue;
        }
        let details = change
            .details
            .iter()
            .filter(|detail| {
                if detail.starts_with("moved") {
                    kinds.contains(&LayoutChangeKind::Moved)
                } else if detail.contains('×') {
                    kinds.contains(&LayoutChangeKind::Resized)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        roots.push(LayoutChange {
            kinds,
            details,
            ..change.clone()
        });
    }
    let top = |change: &LayoutChange| {
        change
            .before_boxes
            .first()
            .map(|index| before.boxes[*index].rect.y)
            .or_else(|| {
                change
                    .after_boxes
                    .first()
                    .map(|index| after.boxes[*index].rect.y)
            })
            .unwrap_or(0)
    };
    roots.sort_by(|a, b| top(a).cmp(&top(b)).then_with(|| a.path.cmp(&b.path)));
    LayoutDiff {
        changes: roots,
        consequences,
    }
}

struct Sides<'a> {
    before: &'a PageLayout,
    before_tables: &'a LayoutTables,
    after: &'a PageLayout,
    after_tables: &'a LayoutTables,
    before_by_path: &'a HashMap<String, Vec<usize>>,
    after_by_path: &'a HashMap<String, Vec<usize>>,
}

type Found = Option<(Vec<LayoutChangeKind>, Vec<String>)>;

fn signed_delta(value: i32) -> String {
    if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

impl Sides<'_> {
    fn compare_single(&self, before: Option<usize>, after: Option<usize>) -> Found {
        let (before_index, after_index) = match (before, after) {
            (None, None) => return None,
            (Some(_), None) => return Some((vec![LayoutChangeKind::Removed], Vec::new())),
            (None, Some(_)) => return Some((vec![LayoutChangeKind::Added], Vec::new())),
            (Some(before), Some(after)) => (before, after),
        };
        let node = &self.before.boxes[before_index];
        let other = &self.after.boxes[after_index];
        let mut kinds = Vec::new();
        let mut details = Vec::new();
        let relative = |layout: &PageLayout, layout_box: &LayoutBox| {
            let parent = layout_box
                .parent
                .and_then(|parent| layout.boxes.get(parent))
                .map_or((0, 0), |parent| (parent.rect.x, parent.rect.y));
            (layout_box.rect.x - parent.0, layout_box.rect.y - parent.1)
        };
        let a = relative(self.before, node);
        let b = relative(self.after, other);
        if (a.0 - b.0).abs() > TOLERANCE || (a.1 - b.1).abs() > TOLERANCE {
            kinds.push(LayoutChangeKind::Moved);
            details.push(format!(
                "moved {}, {}",
                signed_delta(b.0 - a.0),
                signed_delta(b.1 - a.1)
            ));
        }
        if (node.rect.width - other.rect.width).abs() > TOLERANCE
            || (node.rect.height - other.rect.height).abs() > TOLERANCE
        {
            kinds.push(LayoutChangeKind::Resized);
            details.push(format!(
                "{}×{} → {}×{}",
                node.rect.width, node.rect.height, other.rect.width, other.rect.height
            ));
        }
        let style_before = self.before_tables.style(node.style);
        let style_after = self.after_tables.style(other.style);
        let style_changes: Vec<String> = STYLE_PROPS
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| {
                let value_before = style_before.get(index)?;
                let value_after = style_after.get(index)?;
                (value_before != value_after)
                    .then(|| format!("{prop} {value_before} → {value_after}"))
            })
            .collect();
        if !style_changes.is_empty() {
            kinds.push(LayoutChangeKind::Restyled);
            details.extend(style_changes);
        }
        if node.text_hash != other.text_hash {
            kinds.push(LayoutChangeKind::Text);
            details.push("text changed".into());
        }
        if node.image_hash != other.image_hash {
            kinds.push(LayoutChangeKind::Image);
            details.push("image source changed".into());
        }
        (!kinds.is_empty()).then_some((kinds, details))
    }

    /// Compares every box at one list path. Only what most items agree on
    /// counts, so a product swapped for another, or a badge on one product,
    /// does not show up as a layout change.
    fn compare_list(&self, path: &str, before: &[usize], after: &[usize]) -> Found {
        let item_path = path
            .rfind("#*")
            .map_or(path, |position| &path[..position + 2]);
        let items_before = self.before_by_path.get(item_path).map_or(0, Vec::len);
        let items_after = self.after_by_path.get(item_path).map_or(0, Vec::len);
        let share = |count: usize, items: usize| {
            if items == 0 {
                0.0
            } else {
                count as f64 / items as f64
            }
        };
        let is_item = path == item_path;
        if is_item && before.len() != after.len() {
            if before.is_empty() {
                return Some((
                    vec![LayoutChangeKind::Added],
                    vec![format!("{} items", after.len())],
                ));
            }
            if after.is_empty() {
                return Some((
                    vec![LayoutChangeKind::Removed],
                    vec![format!("{} items", before.len())],
                ));
            }
        }
        if !is_item {
            if before.is_empty() {
                return (share(after.len(), items_after) >= 0.5)
                    .then(|| (vec![LayoutChangeKind::Added], vec!["in most items".into()]));
            }
            if after.is_empty() {
                return (share(before.len(), items_before) >= 0.5).then(|| {
                    (
                        vec![LayoutChangeKind::Removed],
                        vec!["from most items".into()],
                    )
                });
            }
        }
        if before.is_empty() || after.is_empty() {
            return None;
        }

        let mut kinds = Vec::new();
        let mut details = Vec::new();
        if is_item && before.len() != after.len() {
            kinds.push(LayoutChangeKind::Count);
            details.push(format!("{} → {} items", before.len(), after.len()));
        }
        let typical = |layout: &PageLayout, boxes: &[usize], field: fn(&Rect) -> i32| {
            mode(boxes.iter().map(|index| {
                (field(&layout.boxes[*index].rect) as f64 / TOLERANCE as f64).round() as i64
            }))
        };
        let width_before = typical(self.before, before, |rect| rect.width);
        let width_after = typical(self.after, after, |rect| rect.width);
        let height_before = typical(self.before, before, |rect| rect.height);
        let height_after = typical(self.after, after, |rect| rect.height);
        let agreed_change =
            |a: &Option<(i64, f64)>, b: &Option<(i64, f64)>, distance: i64| match (a, b) {
                (Some((value_a, share_a)), Some((value_b, share_b))) => {
                    *share_a >= 0.5 && *share_b >= 0.5 && (value_a - value_b).abs() > distance
                }
                _ => false,
            };
        if agreed_change(&width_before, &width_after, 1)
            || agreed_change(&height_before, &height_after, 1)
        {
            kinds.push(LayoutChangeKind::Resized);
            let size = |width: &Option<(i64, f64)>, height: &Option<(i64, f64)>| {
                format!(
                    "{}×{}",
                    width.map_or(0, |(value, _)| value * i64::from(TOLERANCE)),
                    height.map_or(0, |(value, _)| value * i64::from(TOLERANCE))
                )
            };
            details.push(format!(
                "typical {} → {}",
                size(&width_before, &height_before),
                size(&width_after, &height_after)
            ));
        }
        let mut style_changes = Vec::new();
        for (index, prop) in STYLE_PROPS.iter().enumerate() {
            let value_before = mode_str(before.iter().filter_map(|node| {
                self.before_tables
                    .style(self.before.boxes[*node].style)
                    .get(index)
                    .map(String::as_str)
            }));
            let value_after = mode_str(after.iter().filter_map(|node| {
                self.after_tables
                    .style(self.after.boxes[*node].style)
                    .get(index)
                    .map(String::as_str)
            }));
            if let (Some((value_before, share_before)), Some((value_after, share_after))) =
                (value_before, value_after)
                && share_before >= 0.5
                && share_after >= 0.5
                && value_before != value_after
            {
                style_changes.push(format!("{prop} {value_before} → {value_after}"));
            }
        }
        if !style_changes.is_empty() {
            kinds.push(LayoutChangeKind::Restyled);
            details.extend(style_changes);
        }
        (!kinds.is_empty()).then_some((kinds, details))
    }
}

/// The most common value and the share of values that are it.
fn mode(values: impl Iterator<Item = i64>) -> Option<(i64, f64)> {
    let mut counts: HashMap<i64, usize> = HashMap::new();
    let mut total = 0;
    for value in values {
        *counts.entry(value).or_default() += 1;
        total += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        .map(|(value, count)| (value, count as f64 / total as f64))
}

fn mode_str<'a>(values: impl Iterator<Item = &'a str>) -> Option<(&'a str, f64)> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut total = 0;
    for value in values {
        *counts.entry(value).or_default() += 1;
        total += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(value, count)| (value, count as f64 / total as f64))
}

/// Layouts read from the database at a time by passes over a whole crawl.
const BATCH: i64 = 200;

/// Reads every stored layout of the crawl in URL order, `BATCH` at a time,
/// so a pass over a large crawl never holds every layout at once.
pub(crate) async fn for_each_layout(
    pool: &sqlx::SqlitePool,
    crawl_id: i64,
    mut visit: impl FnMut(String, PageLayout),
) -> Result<(), sqlx::Error> {
    let mut after = String::new();
    loop {
        let rows = crate::storage::load_page_layouts_after(pool, crawl_id, &after, BATCH).await?;
        let Some((last, _)) = rows.last() else {
            return Ok(());
        };
        after = last.clone();
        for (url, encoded) in rows {
            match PageLayout::decode(&encoded) {
                Ok(layout) => visit(url, layout),
                Err(e) => tracing::warn!(%url, error=%e, "stored layout could not be decoded"),
            }
        }
    }
}

/// What changed in one page's layout since the baseline crawl.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageLayoutChange {
    pub layout_changes: usize,
    pub content_changes: usize,
    /// The layout changes in a line, such as `h1.title resized, restyled`.
    pub summary: String,
}

/// Compares every page both crawls captured. Only pages whose layout
/// changed are returned: a page whose text or images changed but whose
/// layout did not is unchanged here.
pub async fn compare_crawls(
    pool: &sqlx::SqlitePool,
    baseline_crawl_id: i64,
    crawl_id: i64,
) -> Result<HashMap<String, PageLayoutChange>, sqlx::Error> {
    let baseline_tables = crate::storage::load_layout_tables(pool, baseline_crawl_id).await?;
    let tables = crate::storage::load_layout_tables(pool, crawl_id).await?;
    let mut changed = HashMap::new();
    let mut after = String::new();
    loop {
        let rows = crate::storage::load_page_layouts_after(pool, crawl_id, &after, BATCH).await?;
        let Some((last, _)) = rows.last() else {
            return Ok(changed);
        };
        after = last.clone();
        for (url, encoded) in rows {
            let Some(baseline_encoded) =
                crate::storage::load_page_layout(pool, baseline_crawl_id, &url).await?
            else {
                continue;
            };
            let (layout, baseline) = match (
                PageLayout::decode(&encoded),
                PageLayout::decode(&baseline_encoded),
            ) {
                (Ok(layout), Ok(baseline)) => (layout, baseline),
                (Err(e), _) | (_, Err(e)) => {
                    tracing::warn!(%url, error=%e, "stored layout could not be decoded");
                    continue;
                }
            };
            let diff = diff_layouts(&baseline, &baseline_tables, &layout, &tables);
            let layout_changes = diff.layout_change_count();
            if layout_changes > 0 {
                changed.insert(
                    url,
                    PageLayoutChange {
                        layout_changes,
                        content_changes: diff.content_change_count(),
                        summary: diff.summary(),
                    },
                );
            }
        }
    }
}

/// One page's layout in two crawls and what changed between them, for the
/// wireframe view.
pub struct PageComparison {
    pub before: Option<(PageLayout, LayoutTables)>,
    pub after: (PageLayout, LayoutTables),
    pub diff: LayoutDiff,
}

/// Loads one page's layout, and its baseline when there is one. `None` when
/// the crawl did not capture the page.
pub async fn load_page_comparison(
    pool: &sqlx::SqlitePool,
    baseline_crawl_id: Option<i64>,
    crawl_id: i64,
    url: &str,
) -> anyhow::Result<Option<PageComparison>> {
    let Some(encoded) = crate::storage::load_page_layout(pool, crawl_id, url).await? else {
        return Ok(None);
    };
    let after = (
        PageLayout::decode(&encoded)?,
        crate::storage::load_layout_tables(pool, crawl_id).await?,
    );
    let mut before = None;
    if let Some(baseline_crawl_id) = baseline_crawl_id
        && let Some(encoded) =
            crate::storage::load_page_layout(pool, baseline_crawl_id, url).await?
    {
        before = Some((
            PageLayout::decode(&encoded)?,
            crate::storage::load_layout_tables(pool, baseline_crawl_id).await?,
        ));
    }
    let diff = match &before {
        Some((layout, tables)) => diff_layouts(layout, tables, &after.0, &after.1),
        None => LayoutDiff::default(),
    };
    Ok(Some(PageComparison {
        before,
        after,
        diff,
    }))
}

/// A page's shape for template grouping: the area weight of each grouping
/// path (the last `PATH_WINDOW` keys), and the grouping paths of the
/// ancestors of every box with that path.
#[derive(Clone, Debug, Default)]
pub struct PageShape {
    pub weights: HashMap<String, f64>,
    pub ancestors: HashMap<String, HashSet<String>>,
}

pub fn page_shape(layout: &PageLayout, tables: &LayoutTables) -> PageShape {
    let mut grouping_paths: Vec<String> = Vec::with_capacity(layout.boxes.len());
    let mut key_chains: Vec<Vec<u32>> = Vec::with_capacity(layout.boxes.len());
    let mut shape = PageShape::default();
    for (index, layout_box) in layout.boxes.iter().enumerate() {
        let mut chain = layout_box
            .parent
            .and_then(|parent| key_chains.get(parent))
            .map(|chain| {
                let start = chain.len().saturating_sub(PATH_WINDOW - 1);
                chain[start..].to_vec()
            })
            .unwrap_or_default();
        chain.push(layout_box.key);
        let path = chain
            .iter()
            .map(|key| tables.key(*key))
            .collect::<Vec<_>>()
            .join(">");
        let weight = layout_box.rect.area().sqrt();
        let entry = shape.weights.entry(path.clone()).or_insert(0.0);
        *entry = entry.max(weight);
        let ancestors = shape.ancestors.entry(path.clone()).or_default();
        let mut parent = layout_box.parent;
        while let Some(parent_index) = parent {
            if let Some(parent_path) = grouping_paths.get(parent_index) {
                ancestors.insert(parent_path.clone());
            }
            parent = layout
                .boxes
                .get(parent_index)
                .and_then(|parent| parent.parent);
        }
        debug_assert_eq!(grouping_paths.len(), index);
        grouping_paths.push(path);
        key_chains.push(chain);
    }
    shape
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style(color: &str, font_size: &str) -> Vec<String> {
        STYLE_PROPS
            .iter()
            .map(|prop| match *prop {
                "color" => color.to_string(),
                "fontSize" => font_size.to_string(),
                "backgroundColor" => "rgba(0, 0, 0, 0)".to_string(),
                "backgroundImage" => "none".to_string(),
                "borderTopWidth" => "0px".to_string(),
                _ => String::new(),
            })
            .collect()
    }

    /// Builds a layout from (parent, class, rect, style, text) rows.
    struct Builder {
        tables: LayoutTables,
        layout: PageLayout,
    }

    impl Builder {
        fn new() -> Self {
            Self {
                tables: LayoutTables::default(),
                layout: PageLayout {
                    width: 1366,
                    height: 2000,
                    types: Vec::new(),
                    boxes: Vec::new(),
                },
            }
        }

        fn with_tables(tables: LayoutTables) -> Self {
            Self {
                tables,
                ..Self::new()
            }
        }

        fn add(
            &mut self,
            parent: Option<usize>,
            key: &str,
            rect: (i32, i32, i32, i32),
            style_values: Vec<String>,
            text_hash: u32,
        ) -> usize {
            let mut added = NewTableEntries::default();
            let key = self.tables.intern_key(key.to_string(), &mut added);
            let style = self.tables.intern_style(style_values, &mut added);
            self.layout.boxes.push(LayoutBox {
                parent,
                key,
                rect: Rect {
                    x: rect.0,
                    y: rect.1,
                    width: rect.2,
                    height: rect.3,
                },
                style,
                text_hash,
                image_hash: None,
                fixed: false,
                clip: None,
            });
            self.layout.boxes.len() - 1
        }
    }

    /// A product page: header, title, price, and a grid of `cards` cards.
    fn product_page(
        tables: LayoutTables,
        title_size: &str,
        price_color: &str,
        cards: usize,
        shuffle_text: u32,
    ) -> (PageLayout, LayoutTables) {
        let mut page = Builder::with_tables(tables);
        let body = page.add(None, "body", (0, 0, 1366, 2000), style("black", "16px"), 0);
        page.add(
            Some(body),
            "header.site-header",
            (0, 0, 1366, 80),
            style("black", "16px"),
            0,
        );
        let main = page.add(
            Some(body),
            "main.main",
            (0, 80, 1366, 1500),
            style("black", "16px"),
            0,
        );
        page.add(
            Some(main),
            "h1.title",
            (20, 100, 600, 40),
            style("black", title_size),
            7,
        );
        page.add(
            Some(main),
            "p.price",
            (20, 160, 200, 30),
            style(price_color, "20px"),
            8,
        );
        let grid = page.add(
            Some(main),
            "ul.related-list",
            (20, 300, 1300, 400),
            style("black", "16px"),
            0,
        );
        for card in 0..cards {
            page.add(
                Some(grid),
                "li.card",
                (20 + card as i32 * 210, 300, 200, 300),
                style("black", "16px"),
                100 + card as u32 + shuffle_text,
            );
        }
        (page.layout, page.tables)
    }

    #[test]
    fn keys_keep_the_first_stable_class() {
        assert_eq!(
            normalize_key("DIV", "is-open product-card", None),
            "div.product-card"
        );
        assert_eq!(normalize_key("div", "css-1x2y3z card", None), "div.card");
        assert_eq!(
            normalize_key("span", "Price_root__a8F3k price", None),
            "span.price"
        );
        assert_eq!(normalize_key("div", "aB3dE9", None), "div");
        assert_eq!(normalize_key("li", "item-12345 item", None), "li.item");
        assert_eq!(normalize_key("div", "menu--active menu", None), "div.menu");
        assert_eq!(
            normalize_key("button", "", Some("add-to-cart-42")),
            "button[add-to-cart-#]"
        );
        assert_eq!(
            normalize_key("main", "main main--with-banner", None),
            "main.main"
        );
        assert_eq!(
            normalize_key("div", "product-card--sale", None),
            "div.product-card--sale"
        );
    }

    #[test]
    fn captures_read_into_shared_tables() -> anyhow::Result<()> {
        let props: Vec<&str> = STYLE_PROPS.to_vec();
        let style_values: Vec<String> = style("black", "16px");
        let json = serde_json::json!({
            "props": props,
            "types": ["Product"],
            "width": 1366,
            "height": 900,
            "styles": [style_values],
            "nodes": [
                [-1, "body", "", null, 0, 0, 1366, 900, 0, 0, null, 0, null],
                [0, "img", "hero is-loaded", null, 10, 20, 300, 200, 0, 0, 12345, 0, [10, 20, 300, 100]],
                [0, "h1", "title", null, 10, 240, 600, 40, 0, 99, null, 1, null]
            ]
        });
        let capture: RawCapture = serde_json::from_value(json)?;
        let mut tables = LayoutTables::default();
        let (layout, added) = PageLayout::from_capture(capture, &mut tables)?;
        assert_eq!(layout.types, ["Product"]);
        assert_eq!(added.keys.len(), 3);
        assert_eq!(added.styles.len(), 1);
        assert_eq!(tables.key(layout.boxes[1].key), "img.hero");
        assert_eq!(layout.boxes[1].image_hash, Some(12345));
        assert!(layout.boxes[2].fixed);
        assert_eq!(layout.boxes[1].parent, Some(0));

        let decoded = PageLayout::decode(&layout.encode()?)?;
        assert_eq!(decoded, layout);
        Ok(())
    }

    #[test]
    fn a_capture_with_unexpected_style_properties_is_refused() {
        let json = serde_json::json!({
            "props": ["color"], "types": [], "width": 1, "height": 1, "styles": [], "nodes": []
        });
        let capture: RawCapture = serde_json::from_value(json).expect("valid shape");
        assert!(PageLayout::from_capture(capture, &mut LayoutTables::default()).is_err());
    }

    #[test]
    fn corrupt_stored_layouts_are_errors() {
        assert!(PageLayout::decode(b"not deflate").is_err());
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(&[ENCODING_VERSION, 10, 10, 0, 5])
            .expect("write");
        let truncated = encoder.finish().expect("finish");
        assert!(PageLayout::decode(&truncated).is_err());
    }

    #[test]
    fn stashed_captures_are_taken_out_of_the_html() {
        let html = br#"<html><head><title>T</title><script id="__sr_layout" type="application/json">{"a":"\u003c/script>"}</script></head><body>B</body></html>"#;
        let (cleaned, json) = take_stashed_capture(html).expect("a stash");
        assert_eq!(
            String::from_utf8(cleaned).expect("utf8"),
            "<html><head><title>T</title></head><body>B</body></html>"
        );
        let value: serde_json::Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["a"], "</script>");
        assert!(take_stashed_capture(b"<html><body>no stash</body></html>").is_none());
    }

    #[test]
    fn identical_layouts_have_no_changes() {
        let (before, tables) = product_page(LayoutTables::default(), "32px", "black", 5, 0);
        let diff = diff_layouts(&before, &tables, &before, &tables);
        assert!(diff.changes.is_empty());
    }

    #[test]
    fn reordered_list_content_is_not_a_change() {
        let (before, tables) = product_page(LayoutTables::default(), "32px", "black", 5, 0);
        let (after, tables) = product_page(tables, "32px", "black", 5, 3);
        let diff = diff_layouts(&before, &tables, &after, &tables);
        assert_eq!(diff.changes.len(), 0, "{:?}", diff.changes);
    }

    #[test]
    fn restyles_are_found_and_named() {
        let (before, tables) = product_page(LayoutTables::default(), "32px", "black", 5, 0);
        let (after, tables) = product_page(tables, "44px", "red", 5, 0);
        let diff = diff_layouts(&before, &tables, &after, &tables);
        let labels: Vec<&str> = diff.layout_changes().map(LayoutChange::label).collect();
        assert_eq!(labels, ["h1.title", "p.price"]);
        assert!(diff.changes[0].details[0].contains("fontSize 32px → 44px"));
        assert_eq!(diff.summary(), "h1.title restyled; p.price restyled");
    }

    #[test]
    fn list_counts_and_text_changes_are_told_apart() {
        let (before, tables) = product_page(LayoutTables::default(), "32px", "black", 5, 0);
        let (mut after, tables) = product_page(tables, "32px", "black", 4, 0);
        after.boxes[3].text_hash = 70;
        let diff = diff_layouts(&before, &tables, &after, &tables);
        let list = diff
            .changes
            .iter()
            .find(|change| change.label() == "li.card")
            .expect("the list change");
        assert_eq!(list.kinds, [LayoutChangeKind::Count]);
        assert_eq!(list.before_boxes.len(), 5);
        let title = diff
            .changes
            .iter()
            .find(|change| change.label() == "h1.title")
            .expect("the text change");
        assert_eq!(title.kinds, [LayoutChangeKind::Text]);
        assert_eq!(diff.layout_change_count(), 1);
        assert_eq!(diff.content_change_count(), 1);
    }

    #[test]
    fn a_removed_box_hides_the_moves_it_causes() {
        let mut first = Builder::new();
        let body = first.add(None, "body", (0, 0, 1366, 600), style("black", "16px"), 0);
        first.add(
            Some(body),
            "nav.breadcrumbs",
            (0, 0, 1366, 40),
            style("black", "16px"),
            1,
        );
        first.add(
            Some(body),
            "h1.title",
            (0, 40, 600, 40),
            style("black", "32px"),
            2,
        );
        first.add(
            Some(body),
            "p.price",
            (0, 80, 200, 30),
            style("black", "20px"),
            3,
        );
        let mut second = Builder::with_tables(first.tables.clone());
        let body = second.add(None, "body", (0, 0, 1366, 560), style("black", "16px"), 0);
        second.add(
            Some(body),
            "h1.title",
            (0, 0, 600, 40),
            style("black", "32px"),
            2,
        );
        second.add(
            Some(body),
            "p.price",
            (0, 40, 200, 30),
            style("black", "20px"),
            3,
        );

        let diff = diff_layouts(&first.layout, &first.tables, &second.layout, &second.tables);
        let labels: Vec<String> = diff
            .changes
            .iter()
            .map(|change| format!("{} {}", change.label(), change.kinds_text()))
            .collect();
        assert_eq!(labels, ["nav.breadcrumbs removed"]);
        assert!(diff.consequences >= 2, "the moves and the body resize");
    }

    #[test]
    fn shapes_window_paths_and_record_ancestors() {
        let (page, tables) = product_page(LayoutTables::default(), "32px", "black", 5, 0);
        let shape = page_shape(&page, &tables);
        assert!(
            shape
                .weights
                .contains_key("body>main.main>ul.related-list>li.card")
        );
        let card_ancestors = &shape.ancestors["body>main.main>ul.related-list>li.card"];
        assert!(card_ancestors.contains("body>main.main>ul.related-list"));
        assert!(card_ancestors.contains("body"));
        assert_eq!(shape.weights.len(), 7, "the five cards share one path");
    }

    #[test]
    fn wrappers_do_not_paint() {
        let mut page = Builder::new();
        let wrapper = page.add(
            None,
            "div.wrapper",
            (0, 0, 100, 100),
            style("black", "16px"),
            0,
        );
        let text = page.add(
            Some(wrapper),
            "p.text",
            (0, 0, 100, 20),
            style("black", "16px"),
            5,
        );
        let image = page.add(
            Some(wrapper),
            "img.hero",
            (0, 20, 100, 50),
            style("black", "16px"),
            0,
        );
        assert!(!paints(&page.layout.boxes[wrapper], &page.tables));
        assert!(paints(&page.layout.boxes[text], &page.tables));
        assert!(paints(&page.layout.boxes[image], &page.tables));
    }
}
