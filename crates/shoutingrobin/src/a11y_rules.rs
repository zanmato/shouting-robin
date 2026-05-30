pub fn rule_description(rule_id: &str) -> Option<&'static str> {
    match rule_id {
        "area-alt" => Some("Ensure <area> elements of image maps have alternative text"),
        "aria-allowed-attr" => Some("Ensure an element's role supports its ARIA attributes"),
        "aria-braille-equivalent" => Some(
            "Ensure aria-braillelabel and aria-brailleroledescription have a non-braille equivalent",
        ),
        "aria-command-name" => {
            Some("Ensure every ARIA button, link and menuitem has an accessible name")
        }
        "aria-conditional-attr" => Some(
            "Ensure ARIA attributes are used as described in the specification of the element's role",
        ),
        "aria-deprecated-role" => Some("Ensure elements do not use deprecated roles"),
        "aria-hidden-body" => {
            Some("Ensure aria-hidden=\"true\" is not present on the document body.")
        }
        "aria-hidden-focus" => {
            Some("Ensure aria-hidden elements are not focusable nor contain focusable elements")
        }
        "aria-input-field-name" => Some("Ensure every ARIA input field has an accessible name"),
        "aria-meter-name" => Some("Ensure every ARIA meter node has an accessible name"),
        "aria-progressbar-name" => {
            Some("Ensure every ARIA progressbar node has an accessible name")
        }
        "aria-prohibited-attr" => {
            Some("Ensure ARIA attributes are not prohibited for an element's role")
        }
        "aria-required-attr" => {
            Some("Ensure elements with ARIA roles have all required ARIA attributes")
        }
        "aria-required-children" => {
            Some("Ensure elements with an ARIA role that require child roles contain them")
        }
        "aria-required-parent" => Some(
            "Ensure elements with an ARIA role that require parent roles are contained by them",
        ),
        "aria-roles" => Some("Ensure all elements with a role attribute use a valid value"),
        "aria-tab-name" => Some("Ensure every ARIA tab node has an accessible name"),
        "aria-toggle-field-name" => Some("Ensure every ARIA toggle field has an accessible name"),
        "aria-tooltip-name" => Some("Ensure every ARIA tooltip node has an accessible name"),
        "aria-valid-attr-value" => Some("Ensure all ARIA attributes have valid values"),
        "aria-valid-attr" => {
            Some("Ensure attributes that begin with aria- are valid ARIA attributes")
        }
        "blink" => Some("Ensure <blink> elements are not used"),
        "button-name" => Some("Ensure buttons have discernible text"),
        "bypass" => Some(
            "Ensure each page has at least one mechanism for a user to bypass navigation and jump straight to the content",
        ),
        "color-contrast" => Some(
            "Ensure the contrast between foreground and background colors meets WCAG 2 AA minimum contrast ratio thresholds",
        ),
        "definition-list" => Some("Ensure <dl> elements are structured correctly"),
        "dlitem" => Some("Ensure <dt> and <dd> elements are contained by a <dl>"),
        "document-title" => Some("Ensure each HTML document contains a non-empty <title> element"),
        "duplicate-id-aria" => {
            Some("Ensure every id attribute value used in ARIA and in labels is unique")
        }
        "form-field-multiple-labels" => {
            Some("Ensure form field does not have multiple label elements")
        }
        "frame-focusable-content" => Some(
            "Ensure <frame> and <iframe> elements with focusable content do not have tabindex=-1",
        ),
        "frame-title-unique" => {
            Some("Ensure <iframe> and <frame> elements contain a unique title attribute")
        }
        "frame-title" => Some("Ensure <iframe> and <frame> elements have an accessible name"),
        "html-has-lang" => Some("Ensure every HTML document has a lang attribute"),
        "html-lang-valid" => {
            Some("Ensure the lang attribute of the <html> element has a valid value")
        }
        "html-xml-lang-mismatch" => Some(
            "Ensure that HTML elements with both valid lang and xml:lang attributes agree on the base language of the page",
        ),
        "image-alt" => {
            Some("Ensure <img> elements have alternative text or a role of none or presentation")
        }
        "input-button-name" => Some("Ensure input buttons have discernible text"),
        "input-image-alt" => Some("Ensure <input type=\"image\"> elements have alternative text"),
        "label" => Some("Ensure every form element has a label"),
        "link-in-text-block" => Some(
            "Ensure links are distinguished from surrounding text in a way that does not rely on color",
        ),
        "link-name" => Some("Ensure links have discernible text"),
        "list" => Some("Ensure that lists are structured correctly"),
        "listitem" => Some("Ensure <li> elements are used semantically"),
        "marquee" => Some("Ensure <marquee> elements are not used"),
        "meta-refresh" => {
            Some("Ensure <meta http-equiv=\"refresh\"> is not used for delayed refresh")
        }
        "meta-viewport" => {
            Some("Ensure <meta name=\"viewport\"> does not disable text scaling and zooming")
        }
        "nested-interactive" => Some(
            "Ensure interactive controls are not nested as they are not always announced by screen readers or can cause focus problems for assistive technologies",
        ),
        "no-autoplay-audio" => Some(
            "Ensure <video> or <audio> elements do not autoplay audio for more than 3 seconds without a control mechanism to stop or mute the audio",
        ),
        "object-alt" => Some("Ensure <object> elements have alternative text"),
        "role-img-alt" => Some("Ensure [role=\"img\"] elements have alternative text"),
        "scrollable-region-focusable" => Some(
            "Ensure elements that have scrollable content are accessible by keyboard in Safari",
        ),
        "select-name" => Some("Ensure select element has an accessible name"),
        "server-side-image-map" => Some("Ensure that server-side image maps are not used"),
        "summary-name" => Some("Ensure summary elements have discernible text"),
        "svg-img-alt" => Some(
            "Ensure <svg> elements with an img, graphics-document or graphics-symbol role have accessible text",
        ),
        "td-headers-attr" => Some(
            "Ensure that each cell in a table that uses the headers attribute refers only to other <th> elements in that table",
        ),
        "th-has-data-cells" => Some(
            "Ensure that <th> elements and elements with role=columnheader/rowheader have data cells they describe",
        ),
        "valid-lang" => Some("Ensure lang attributes have valid values"),
        "video-caption" => Some("Ensure <video> elements have captions"),
        "autocomplete-valid" => {
            Some("Ensure the autocomplete attribute is correct and suitable for the form field")
        }
        "avoid-inline-spacing" => Some(
            "Ensure that text spacing set through style attributes can be adjusted with custom stylesheets",
        ),
        "target-size" => Some("Ensure touch targets have sufficient size and space"),
        "accesskeys" => Some("Ensure every accesskey attribute value is unique"),
        "aria-allowed-role" => {
            Some("Ensure role attribute has an appropriate value for the element")
        }
        "aria-dialog-name" => {
            Some("Ensure every ARIA dialog and alertdialog node has an accessible name")
        }
        "aria-text" => {
            Some("Ensure role=\"text\" is used on elements with no focusable descendants")
        }
        "aria-treeitem-name" => Some("Ensure every ARIA treeitem node has an accessible name"),
        "empty-heading" => Some("Ensure headings have discernible text"),
        "empty-table-header" => Some("Ensure table headers have discernible text"),
        "frame-tested" => Some("Ensure <iframe> and <frame> elements contain the axe-core script"),
        "heading-order" => Some("Ensure the order of headings is semantically correct"),
        "image-redundant-alt" => Some("Ensure image alternative is not repeated as text"),
        "label-title-only" => Some(
            "Ensure that every form element has a visible label and is not solely labeled using hidden labels, or the title or aria-describedby attributes",
        ),
        "landmark-banner-is-top-level" => Some("Ensure the banner landmark is at top level"),
        "landmark-contentinfo-is-top-level" => {
            Some("Ensure the contentinfo landmark is at top level")
        }
        "landmark-main-is-top-level" => Some("Ensure the main landmark is at top level"),
        "landmark-no-duplicate-banner" => {
            Some("Ensure the document has at most one banner landmark")
        }
        "landmark-no-duplicate-contentinfo" => {
            Some("Ensure the document has at most one contentinfo landmark")
        }
        "landmark-no-duplicate-main" => Some("Ensure the document has at most one main landmark"),
        "landmark-one-main" => Some("Ensure the document has a main landmark"),
        "landmark-unique" => Some("Ensure landmarks are unique"),
        "meta-viewport-large" => {
            Some("Ensure <meta name=\"viewport\"> can scale a significant amount")
        }
        "page-has-heading-one" => {
            Some("Ensure that the page, or at least one of its frames contains a level-one heading")
        }
        "presentation-role-conflict" => Some(
            "Ensure elements marked as presentational do not have global ARIA or tabindex so that all screen readers ignore them",
        ),
        "region" => Some("Ensure all page content is contained by landmarks"),
        "scope-attr-valid" => Some("Ensure the scope attribute is used correctly on tables"),
        "skip-link" => Some("Ensure all skip links have a focusable target"),
        "tabindex" => Some("Ensure tabindex attribute values are not greater than 0"),
        "table-duplicate-name" => Some(
            "Ensure the <caption> element does not contain the same text as the summary attribute",
        ),
        "color-contrast-enhanced" => Some(
            "Ensure the contrast between foreground and background colors meets WCAG 2 AAA enhanced contrast ratio thresholds",
        ),
        "identical-links-same-purpose" => {
            Some("Ensure that links with the same accessible name serve a similar purpose")
        }
        "meta-refresh-no-exceptions" => {
            Some("Ensure <meta http-equiv=\"refresh\"> is not used for delayed refresh")
        }
        "css-orientation-lock" => Some(
            "Ensure content is not locked to any specific display orientation, and the content is operable in all display orientations",
        ),
        "focus-order-semantics" => Some(
            "Ensure elements in the focus order have a role appropriate for interactive content",
        ),
        "hidden-content" => Some("Inform users about hidden content."),
        "label-content-name-mismatch" => Some(
            "Ensure that elements labelled through their content must have their visible text as part of their accessible name",
        ),
        "p-as-heading" => Some(
            "Ensure bold, italic text and font-size is not used to style <p> elements as a heading",
        ),
        "table-fake-caption" => {
            Some("Ensure that tables with a caption use the <caption> element.")
        }
        "td-has-header" => Some(
            "Ensure that each non-empty data cell in a <table> larger than 3 by 3 has one or more table headers",
        ),
        "aria-roledescription" => Some(
            "Ensure aria-roledescription is only used on elements with an implicit or explicit role",
        ),
        "audio-caption" => Some("Ensure <audio> elements have captions"),
        "duplicate-id-active" => {
            Some("Ensure every id attribute value of active elements is unique")
        }
        "duplicate-id" => Some("Ensure every id attribute value is unique"),
        "landmark-complementary-is-top-level" => {
            Some("Ensure the complementary landmark or aside is at top level")
        }
        _ => None,
    }
}
