use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

pub static PROP_ALIASES_OBJ: Lazy<HashMap<&str, HashMap<&str, &str>>> = Lazy::new(|| {
    HashMap::from([
        (
            "formnovalidate",
            HashMap::from([("$", "formNoValidate"), ("BUTTON", "1"), ("INPUT", "1")]),
        ),
        ("ismap", HashMap::from([("$", "isMap"), ("IMG", "1")])),
        (
            "nomodule",
            HashMap::from([("$", "noModule"), ("SCRIPT", "1")]),
        ),
        (
            "playsinline",
            HashMap::from([("$", "playsInline"), ("VIDEO", "1")]),
        ),
        (
            "readonly",
            HashMap::from([("$", "readOnly"), ("INPUT", "1"), ("TEXTAREA", "1")]),
        ),
    ])
});

pub fn get_prop_alias(prop: &str, tag_name: &str) -> Option<String> {
    if prop == "class" {
        return Some("className".to_string());
    }
    let a = PROP_ALIASES_OBJ.get(prop)?;
    a.get(tag_name).map(|_| a.get("$").unwrap().to_string())
}

pub static DELEGATED_EVENTS: Lazy<HashSet<&str>> = Lazy::new(|| {
    HashSet::from([
        "beforeinput",
        "click",
        "dblclick",
        "contextmenu",
        "focusin",
        "focusout",
        "input",
        "keydown",
        "keyup",
        "mousedown",
        "mousemove",
        "mouseout",
        "mouseover",
        "mouseup",
        "pointerdown",
        "pointermove",
        "pointerout",
        "pointerover",
        "pointerup",
        "touchend",
        "touchmove",
        "touchstart",
    ])
});

pub const SVG_ELEMENTS: [&str; 76] = [
    "altGlyph",
    "altGlyphDef",
    "altGlyphItem",
    "animate",
    "animateColor",
    "animateMotion",
    "animateTransform",
    "circle",
    "clipPath",
    "color-profile",
    "cursor",
    "defs",
    "desc",
    "ellipse",
    "feBlend",
    "feColorMatrix",
    "feComponentTransfer",
    "feComposite",
    "feConvolveMatrix",
    "feDiffuseLighting",
    "feDisplacementMap",
    "feDistantLight",
    "feFlood",
    "feFuncA",
    "feFuncB",
    "feFuncG",
    "feFuncR",
    "feGaussianBlur",
    "feImage",
    "feMerge",
    "feMergeNode",
    "feMorphology",
    "feOffset",
    "fePointLight",
    "feSpecularLighting",
    "feSpotLight",
    "feTile",
    "feTurbulence",
    "filter",
    "font",
    "font-face",
    "font-face-format",
    "font-face-name",
    "font-face-src",
    "font-face-uri",
    "foreignObject",
    "g",
    "glyph",
    "glyphRef",
    "hkern",
    "image",
    "line",
    "linearGradient",
    "marker",
    "mask",
    "metadata",
    "missing-glyph",
    "mpath",
    "path",
    "pattern",
    "polygon",
    "polyline",
    "radialGradient",
    "rect",
    "set",
    "stop",
    "svg",
    "switch",
    "symbol",
    "text",
    "textPath",
    "tref",
    "tspan",
    "use",
    "view",
    "vkern",
];

pub static SVGNAMESPACE: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("xlink", "http://www.w3.org/1999/xlink"),
        ("xml", "http://www.w3.org/XML/1998/namespace"),
    ])
});

pub const VOID_ELEMENTS: [&str; 16] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "menuitem",
    "meta", "param", "source", "track", "wbr",
];

pub static ALIASES: Lazy<HashMap<&str, &str>> =
    Lazy::new(|| HashMap::from([("className", "class"), ("htmlFor", "for")]));

pub static CHILD_PROPERTIES: Lazy<HashSet<&str>> =
    Lazy::new(|| HashSet::from(["innerHTML", "textContent", "innerText", "children"]));

pub const BOOLEANS: [&str; 24] = [
    "allowfullscreen",
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "default",
    "disabled",
    "formnovalidate",
    "hidden",
    "indeterminate",
    "ismap",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "seamless",
    "selected",
];

pub static PROPERTIES: Lazy<HashSet<&str>> = Lazy::new(|| {
    [
        "className",
        "value",
        "readOnly",
        "formNoValidate",
        "isMap",
        "noModule",
        "playsInline",
    ]
    .into_iter()
    .chain(BOOLEANS)
    .collect()
});
