import { template as _$template } from "r-dom";
import { memo as _$memo } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(
    `<div id="main"><style>div { color: red; }</style><h1>Welcome</h1><label for="entry">Edit:</label><input id="entry" type="text">`
  ),
  _tmpl$2 = /*#__PURE__*/ _$template(`<div><span><a></a></span><span>`),
  _tmpl$3 = /*#__PURE__*/ _$template(`<div><div><table><tbody></tbody></table></div><div>`),
  _tmpl$4 = /*#__PURE__*/ _$template(
    `<div><div><footer><div></div></footer></div><div><button><span>0`
  );
export const template = _tmpl$();
export const template2 = _tmpl$2();
export const template3 = _tmpl$3();
export const template4 = _tmpl$4();
export const template5 = "Hello";
export const template6 = "Hello";
export const template7 = _$memo(()=>props.id);
export const template8 = [
  "1",
  "2"
];
export const template9 = [
  "1",
  _$memo(()=>props.id)
];
export const template10 = 1;
export const template11 = _$memo(()=>`Hello ${props.name}`);
let id = 123;
export const template12 = id;
const signal = ()=>1;
export const template13 = _$memo(signal);
