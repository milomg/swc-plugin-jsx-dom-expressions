import { template as _$template } from "solid-js/web";
import { createComponent as _$createComponent } from "solid-js/web";
const _tmpl$ = /*#__PURE__*/ _$template(`<div>Hello</div>`, 2), _tmpl$2 = /*#__PURE__*/ _$template(`<div>hi</div>`, 2), _tmpl$3 = /*#__PURE__*/ _$template(`<div>wat</div>`, 2);
const a = ()=>_$createComponent(Comp, {
        get children () {
            return [
                _tmpl$.cloneNode(true),
                _tmpl$2.cloneNode(true),
                _tmpl$3.cloneNode(true)
            ];
        }
    });
