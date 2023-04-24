import { template as _$template } from "r-dom";
import { style as _$style } from "r-dom";
import { effect as _$effect } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div><p>hi</p></div>`, 4);
const a = (size)=>{
    const s = {
        "font-size": `${size}px`
    };
    return (()=>{
        const _el$ = _tmpl$.cloneNode(true), _el$1 = _el$.firstChild;
        _el$.style.setProperty("width", size);
        _el$.style.setProperty("height", size * 2);
        _$effect((_$p)=>_$style(_el$1, s, _$p));
        return _el$;
    })();
};
