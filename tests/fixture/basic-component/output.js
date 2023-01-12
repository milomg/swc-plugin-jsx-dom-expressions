import { createComponent as _$createComponent } from "solid-js/web";
const a = () => _$createComponent(Comp, {
  a: "b",
  get foo() {
    return bar();
  }
});
