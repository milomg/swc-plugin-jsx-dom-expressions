import { createComponent as _$createComponent } from "r-dom";
import { Show as _$Show } from "r-dom";
import { For as _$For } from "r-dom";
const Component = _$createComponent(_$For, {
    get each () {
        return state.list;
    },
    get fallback () {
        return _$createComponent(Loading, {});
    },
    children: (item)=>_$createComponent(_$Show, {
            get when () {
                return state.condition;
            },
            children: item
        })
});
