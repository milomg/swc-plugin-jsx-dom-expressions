use std::collections::HashSet;

use once_cell::sync::Lazy;

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

pub const VOID_ELEMENTS: [&str; 16] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "menuitem",
    "meta", "param", "source", "track", "wbr",
];

// // React Compat
// const Aliases = {
//     className: "class",
//     htmlFor: "for"
//   }

// // React Compat
// #[allow(non_snake_case)]
// pub struct Aliases<'a> {
//     className: &'a str,
//     htmlFor: &'a str,
// }

// pub const ALIASES: Aliases = Aliases {
//     className: "class",
//     htmlFor: "for",
// };

// turn it into a hasmap
pub const ALIASES: [(&str, &str); 2] = [("className", "class"), ("htmlFor", "for")];

pub const CHILD_PROPERTIES: [&str; 4] = ["innerHTML", "textContent", "innerText", "children"];

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
    "selected"
];

pub static PROPERTIES: Lazy<HashSet<&str>> = Lazy::new(||{
    ["className",
    "value",
    "readOnly",
    "formNoValidate",
    "isMap",
    "noModule",
    "playsInline"].into_iter().chain(BOOLEANS.into_iter()).collect()
});