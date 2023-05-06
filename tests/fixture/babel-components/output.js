import { use as _$use } from "r-dom";
import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div>Hello `), _tmpl$2 = /*#__PURE__*/ _$template(`<div>`);
import { Show } from "somewhere";
const Child = (props)=>{
    const [s, set] = createSignal();
    return [
        (()=>{
            const _el$ = _tmpl$(), _el$2 = _el$.firstChild;
            const _ref$ = props.ref;
            typeof _ref$ === "function" ? _$use(_ref$, _el$) : props.ref = _el$;
            _$insert(_el$, ()=>props.name, null);
            return _el$;
        })(),
        (()=>{
            const _el$3 = _tmpl$2();
            _$use(set, _el$3);
            _$insert(_el$3, ()=>props.children);
            return _el$3;
        })()
    ];
};
