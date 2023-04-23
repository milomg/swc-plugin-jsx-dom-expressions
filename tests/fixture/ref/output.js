import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div><p></p></div>`, 4);

const a = ()=>{
    let el;

    (()=>{
        const _el$ = _tmpl$.cloneNode(true), ref = el, _el$1 = _el$.firstChild;
        typeof ref == "function" ? ref(_el$) : el = _el$;
        _$insert(_el$1, el.clientWidth);
        return _el$;
    })();
};