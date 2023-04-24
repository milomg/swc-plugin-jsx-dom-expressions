import { template as _$template } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
import { effect as _$effect } from "r-dom";
import { className as _$className } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div><p>hi</p></div>`, 4);
const f = ()=>"a";
const a = ()=>{
    let b = f();
    return (()=>{
        const _el$ = _tmpl$.cloneNode(true), _el$1 = _el$.firstChild;
        _$effect(()=>_$className(_el$, f()));
        _$setAttribute(_el$1, "title", b);
        return _el$;
    })();
};
