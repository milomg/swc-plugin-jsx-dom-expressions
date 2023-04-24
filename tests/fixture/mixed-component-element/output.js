import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { className as _$className } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div>A</div>`, 2), _tmpl$2 = /*#__PURE__*/ _$template(`<div><p>b</p><p>c</p></div>`, 6);
const A = ()=>_tmpl$.cloneNode(true);
const B = (b)=>(()=>{
    const _el$ = _tmpl$2.cloneNode(true), _el$1 = _el$.firstChild, _el$2 = _el$1.nextSibling;
    _$insert(_el$, _$createComponent(A, {}), _el$1);
    _$className(_el$1, b);
    _$insert(_el$, _$createComponent(A, {}), _el$2);
    _$className(_el$2, b);
    return _el$;
})();
