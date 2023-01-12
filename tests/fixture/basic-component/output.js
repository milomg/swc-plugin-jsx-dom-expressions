import { createComponent as _$createComponent } from "r-dom";
const a = () => _$createComponent(Comp, {
  a: "b",
  get foo() {
    return bar();
  }
});
