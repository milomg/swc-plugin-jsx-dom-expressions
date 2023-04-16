import { template as _$template } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div>hi</div>`, 2);
const a = ()=>(()=>{
    const _el$ = _tmpl$.cloneNode(true);
    _el$.addEventListener("click", ()=>console.log("a"));
    return _el$;
})();