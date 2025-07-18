const selected = true;
let id = "my-h1";
let link;
export const template = (
  <div id="main" {...results} classList={{ selected: unknown }} style={{ color }}>
    <h1
      class="base"
      id={id}
      {...results()}
      foo
      disabled
      title={welcoming()}
      style={{ "background-color": color(), "margin-right": "40px" }}
      classList={{ dynamic: dynamic(), selected }}
    >
      <a href={"/"} ref={link} classList={{ "ccc ddd": true }}>
        Welcome
      </a>
    </h1>
  </div>
);

export const template2 = (
  <div {...getProps("test")}>
    <div textContent={rowId} />
    <div textContent={row.label} />
    <div innerHTML={"<div/>"} />
  </div>
);

export const template3 = (
  <div
    foo
    id={/*@once*/ state.id}
    style={/*@once*/ { "background-color": state.color }}
    name={state.name}
    textContent={/*@once*/ state.content}
  />
);

export const template4 = <div class="hi" className={state.class} classList={{ "ccc:ddd": true }} />;

export const template5 = <div class="a" className="b"></div>;

export const template6 = <div style={someStyle()} textContent="Hi" />;

let undefVar;
export const template7 = (
  <div
    style={{ "background-color": color(), "margin-right": "40px", ...props.style }}
    style:padding-top={props.top}
    class:my-class={props.active}
    class:other-class={undefVar}
    classList={{ 'other-class2': undefVar}}
  />
);

let refTarget;
export const template8 = <div ref={refTarget} />;

export const template9 = <div ref={e => console.log(e)} />;

export const template10 = <div ref={refFactory()} />;

export const template11 = <div use:something use:another={thing} use:zero={0} />;

export const template12 = <div prop:htmlFor={thing} />;

export const template13 = <input type="checkbox" checked={true} />;

export const template14 = <input type="checkbox" checked={state.visible} />;

export const template15 = <div class="`a">`$`</div>;

export const template16 = (
  <button
    class="static"
    classList={{
      hi: "k"
    }}
    type="button"
  >
    Write
  </button>
);

export const template17 = (
  <button
    classList={{
      a: true,
      b: true,
      c: true
    }}
    onClick={increment}
  >
    Hi
  </button>
);

export const template18 = (
  <div
    {...{
      get [key()]() {
        return props.value;
      }
    }}
  />
);

export const template19 = <div classList={{ "bg-red-500": true }} class="flex flex-col" />;

export const template20 = (
  <div>
    <input value={s()} min={min()} max={max()} onInput={doSomething} readonly="" />
    <input checked={s2()} min={min()} max={max()} onInput={doSomethingElse} readonly={value} />
  </div>
);

export const template21 = <div style={{ a: "static", ...rest }}></div>;

export const template22 = <div data='"hi"' data2={'"'} />;

export const template23 = <div disabled={"t" in test}>{"t" in test && "true"}</div>;

export const template24 = <a {...props} something />;

export const template25 = (
  <div>
    {props.children}
    <a {...props} something />
  </div>
);

export const template26 = (
  <div start="Hi" middle={middle} {...spread}>
    Hi
  </div>
);

export const template27 = (
  <div start="Hi" {...first} middle={middle} {...second}>
    Hi
  </div>
);

export const template28 = (
  <label {...api()}>
    <span {...api()}>Input is {api() ? "checked" : "unchecked"}</span>
    <input {...api()} />
    <div {...api()} />
  </label>
);

export const template29 = <div attribute={!!someValue}>{!!someValue}</div>;

export const template30 = (
  <div
    class="class1 class2
    class3 class4
    class5 class6"
    style="color: red;
    background-color: blue !important;
    border: 1px solid black;
    font-size: 12px;"
    random="random1 random2
    random3 random4"
  />
);

export const template31 = (
  <div
    style={{ "background-color": getStore.itemProperties.color }}
  />
);

export const template32 = (
  <div
    style={{ "background-color": undefined }}
  />
);
