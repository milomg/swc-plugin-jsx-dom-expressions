import { template as _$template } from "r-dom";
import { style as _$style } from "r-dom";
import { effect as _$effect } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div>hi</div>`, 2);
const a = (size)=>(()=>{
    const _el$ = _tmpl$.cloneNode(true);
    _$effect((_$p)=>_$style(_el$, {
        width: size,
        height: size * 2
    }, _$p));
    return _el$;
})();
