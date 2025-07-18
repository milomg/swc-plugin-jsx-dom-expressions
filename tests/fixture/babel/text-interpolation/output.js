import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<span>Hello `), _tmpl$2 = /*#__PURE__*/ _$template(`<span> John`), _tmpl$3 = /*#__PURE__*/ _$template(`<span>Hello John`), _tmpl$4 = /*#__PURE__*/ _$template(`<span> `), _tmpl$5 = /*#__PURE__*/ _$template(`<span> <!> <!> `), _tmpl$6 = /*#__PURE__*/ _$template(`<span> <!> `), _tmpl$7 = /*#__PURE__*/ _$template(`<span>Hello`), _tmpl$8 = /*#__PURE__*/ _$template(`<span>&nbsp;&lt;Hi&gt;&nbsp;`), _tmpl$9 = /*#__PURE__*/ _$template(`<span>Hi&lt;script>alert();&lt;/script>`), _tmpl$10 = /*#__PURE__*/ _$template(`<span>4 + 5 = 9`), _tmpl$11 = /*#__PURE__*/ _$template(`<div>
d`), _tmpl$12 = /*#__PURE__*/ _$template(`<div>`), _tmpl$13 = /*#__PURE__*/ _$template(`<div normal="Search…" title="Search&amp;hellip;">`), _tmpl$14 = /*#__PURE__*/ _$template(`<div><div>`);
export const trailing = _tmpl$();
export const leading = _tmpl$2();
/* prettier-ignore */ export const extraSpaces = _tmpl$3();
export const trailingExpr = (()=>{
    const _el$4 = _tmpl$(), _el$5 = _el$4.firstChild;
    _$insert(_el$4, name, null);
    return _el$4;
})();
export const leadingExpr = (()=>{
    const _el$6 = _tmpl$2(), _el$7 = _el$6.firstChild;
    _$insert(_el$6, greeting, _el$7);
    return _el$6;
})();
/* prettier-ignore */ export const multiExpr = (()=>{
    const _el$8 = _tmpl$4(), _el$9 = _el$8.firstChild;
    _$insert(_el$8, greeting, _el$9);
    _$insert(_el$8, name, null);
    return _el$8;
})();
/* prettier-ignore */ export const multiExprSpaced = (()=>{
    const _el$10 = _tmpl$5(), _el$11 = _el$10.firstChild, _el$14 = _el$11.nextSibling, _el$12 = _el$14.nextSibling, _el$15 = _el$12.nextSibling, _el$13 = _el$15.nextSibling;
    _$insert(_el$10, greeting, _el$14);
    _$insert(_el$10, name, _el$15);
    return _el$10;
})();
/* prettier-ignore */ export const multiExprTogether = (()=>{
    const _el$16 = _tmpl$6(), _el$17 = _el$16.firstChild, _el$19 = _el$17.nextSibling, _el$18 = _el$19.nextSibling;
    _$insert(_el$16, greeting, _el$19);
    _$insert(_el$16, name, _el$19);
    return _el$16;
})();
/* prettier-ignore */ export const multiLine = _tmpl$7();
/* prettier-ignore */ export const multiLineTrailingSpace = _tmpl$3();
/* prettier-ignore */ export const multiLineNoTrailingSpace = _tmpl$3();
/* prettier-ignore */ export const escape = _tmpl$8();
/* prettier-ignore */ export const escape2 = _$createComponent(Comp, {
    children: "\xa0<Hi>\xa0"
});
/* prettier-ignore */ export const escape3 = "\xa0<Hi>\xa0";
export const injection = _tmpl$9();
let value = "World";
export const evaluated = (()=>{
    const _el$25 = _tmpl$(), _el$26 = _el$25.firstChild;
    _$insert(_el$25, value + "!", null);
    return _el$25;
})();
let number = 4 + 5;
export const evaluatedNonString = _tmpl$10();
export const newLineLiteral = (()=>{
    const _el$28 = _tmpl$11(), _el$29 = _el$28.firstChild;
    _$insert(_el$28, s, _el$29);
    return _el$28;
})();
export const trailingSpace = (()=>{
    const _el$30 = _tmpl$12();
    _$insert(_el$30, expr);
    return _el$30;
})();
export const trailingSpaceComp = _$createComponent(Comp, {
    children: expr
});
export const trailingSpaceFrag = expr;
export const leadingSpaceElement = (()=>{
    const _el$31 = _tmpl$4(), _el$32 = _el$31.firstChild;
    _$insert(_el$31, expr, null);
    return _el$31;
})();
export const leadingSpaceComponent = _$createComponent(Div, {
    get children () {
        return [
            " ",
            expr
        ];
    }
});
export const leadingSpaceFragment = [
    " ",
    expr
];
export const trailingSpaceElement = (()=>{
    const _el$33 = _tmpl$4(), _el$34 = _el$33.firstChild;
    _$insert(_el$33, expr, _el$34);
    return _el$33;
})();
export const trailingSpaceComponent = _$createComponent(Div, {
    get children () {
        return [
            expr,
            " "
        ];
    }
});
export const trailingSpaceFragment = [
    expr,
    " "
];
export const escapeAttribute = _tmpl$13();
export const escapeCompAttribute = _$createComponent(Div, {
    normal: "Search…",
    title: "Search&hellip;"
});
export const lastElementExpression = (()=>{
    const _el$36 = _tmpl$14(), _el$37 = _el$36.firstChild;
    _$insert(_el$36, expr, null);
    return _el$36;
})();
