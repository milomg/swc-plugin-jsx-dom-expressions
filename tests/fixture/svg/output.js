import { template as _$template } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
const _tmpl$ = /*#__PURE__*/ _$template(`<svg><g><circle r="16" stroke-width="1" stroke="white"></circle></g></svg>`, 6, true);
const a = ({ color , alpha  })=>(()=>{
    const _el$ = _tmpl$.cloneNode(true), _el$1 = _el$.firstChild;
    _$setAttribute(_el$1, "fill", color);
    _$setAttribute(_el$1, "opacity", alpha);
    return _el$;
})();