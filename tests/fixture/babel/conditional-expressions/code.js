export const template1 = <div>{simple}</div>;

export const template2 = <div>{state.dynamic}</div>;

export const template3 = <div>{simple ? good : bad}</div>;

export const template4 = <div>{simple ? good() : bad}</div>;

export const template5 = <div>{state.dynamic ? good() : bad}</div>;

export const template6 = <div>{state.dynamic && good()}</div>;

export const template7 = <div>{state.count > 5 ? (state.dynamic ? best : good()) : bad}</div>;

export const template8 = <div>{state.dynamic && state.something && good()}</div>;

export const template9 = <div>{(state.dynamic && good()) || bad}</div>;

export const template10 = <div>{state.a ? "a" : state.b ? "b" : state.c ? "c" : "fallback"}</div>;

export const template11 = <div>{state.a ? a() : state.b ? b() : state.c ? "c" : "fallback"}</div>;

export const template12 = <Comp render={state.dynamic ? good() : bad} />;

// no dynamic predicate
export const template13 = <Comp render={state.dynamic ? good : bad} />;

export const template14 = <Comp render={state.dynamic && good()} />;

// no dynamic predicate
export const template15 = <Comp render={state.dynamic && good} />;

export const template16 = <Comp render={state.dynamic || good()} />;

export const template17 = <Comp render={state.dynamic ? <Comp /> : <Comp />} />;

export const template18 = <Comp>{state.dynamic ? <Comp /> : <Comp />}</Comp>;

export const template19 = <div innerHTML={state.dynamic ? <Comp /> : <Comp />} />;

export const template20 = <div>{state.dynamic ? <Comp /> : <Comp />}</div>;

export const template21 = <Comp render={state?.dynamic ? "a" : "b"} />;

export const template22 = <Comp>{state?.dynamic ? "a" : "b"}</Comp>;

export const template23 = <div innerHTML={state?.dynamic ? "a" : "b"} />;

export const template24 = <div>{state?.dynamic ? "a" : "b"}</div>;

export const template25 = <Comp render={state.dynamic ?? <Comp />} />;

export const template26 = <Comp>{state.dynamic ?? <Comp />}</Comp>;

export const template27 = <div innerHTML={state.dynamic ?? <Comp />} />;

export const template28 = <div>{state.dynamic ?? <Comp />}</div>;

export const template29 = <div>{(thing() && thing1()) ?? thing2() ?? thing3()}</div>;

export const template30 = <div>{thing() || thing1() || thing2()}</div>;

export const template31 = <Comp value={count() ? (count() ? count() : count()) : count()} />

export const template32 = <div>{something?.()}</div>

export const template33 = <Comp>{something?.()}</Comp>
