import { use as _$use } from "r-dom";
import { template as _$template } from "r-dom";
import { mergeProps as _$mergeProps } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<div>Hello `), _tmpl$2 = /*#__PURE__*/ _$template(`<div>`), _tmpl$3 = /*#__PURE__*/ _$template(`<div>From Parent`);
import { Show } from "somewhere";
const Child = (props)=>{
    const [s, set] = createSignal();
    return [
        (()=>{
            const _el$ = _tmpl$(), _ref$ = props.ref, _el$2 = _el$.firstChild;
            typeof _ref$ === "function" ? _$use(_ref$, _el$) : props.ref = _el$;
            _$insert(_el$, ()=>props.name, null);
            return _el$;
        })(),
        (()=>{
            const _el$3 = _tmpl$2(), _ref$2 = set;
            typeof _ref$2 === "function" ? _$use(_ref$2, _el$3) : set = _el$3;
            _$insert(_el$3, ()=>props.children, null);
            return _el$3;
        })()
    ];
};
const template = (props)=>{
    let childRef;
    const { content  } = props;
    return (()=>{
        const _el$4 = _tmpl$2();
        _$insert(_el$4, _$createComponent(Child, _$mergeProps({
            name: "John"
        }, props, {
            ref (r$) {
                const _ref$3 = childRef;
                typeof _ref$3 === "function" ? _ref$3(r$) : childRef = r$;
            },
            booleanProperty: true,
            get children () {
                return _tmpl$3();
            }
        })), null);
        _$insert(_el$4, _$createComponent(Child, _$mergeProps({
            name: "Jason"
        }, dynamicSpread, {
            ref (r$) {
                const _ref$4 = props.ref;
                typeof _ref$4 === "function" ? _ref$4(r$) : props.ref = r$;
            },
            get children () {
                const _el$6 = _tmpl$2();
                _$insert(_el$6, content, null);
                return _el$6;
            }
        })), null);
        _$insert(_el$4, _$createComponent(Context.Consumer, {
            ref (r$) {
                const _ref$5 = props.consumerRef();
                typeof _ref$5 === "function" && _ref$5(r$);
            },
            children: (context)=>context
        }), null);
        return _el$4;
    })();
};
const template2 = _$createComponent(Child, {
    name: "Jake",
    get dynamic () {
        return state.data;
    },
    stale: state.data,
    handleClick: clickHandler,
    get "hyphen-ated" () {
        return state.data;
    },
    ref: (el)=>e = el
});