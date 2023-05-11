import { Show } from "somewhere"

const Child = props => {
  const [s, set] = createSignal();
  return <>
    <div ref={props.ref}>Hello {props.name}</div>
    <div ref={set}>{props.children}</div>
  </>
};

const template = props => {
  let childRef;
  const { content } = props;
  return (
    <div>
      <Child name="John" {...props} ref={childRef} booleanProperty>
        <div>From Parent</div>
      </Child>
      <Child name="Jason" {...dynamicSpread()} ref={props.ref}>
        {/* Comment Node */}
        <div>{content}</div>
      </Child>
      <Context.Consumer ref={props.consumerRef()}>{context => context}</Context.Consumer>
    </div>
  );
};

const template2 = (
  <Child
    name="Jake"
    dynamic={state.data}
    stale={/*@once*/ state.data}
    handleClick={clickHandler}
    hyphen-ated={state.data}
    ref={el => (e = el)}
  />
);

const template3 = (
  <Child>
    <div />
    <div />
    <div />
    After
  </Child>
);

const [s, set] = createSignal();
const template4 = <Child ref={set}>{<div />}</Child>;

const template5 = <Child dynamic={state.dynamic}>{state.dynamic}</Child>;

// builtIns
const template6 = (
  <For each={state.list} fallback={<Loading />}>
    {item => <Show when={state.condition}>{item}</Show>}
  </For>
);

const template7 = (
  <Child>
    <div />
    {state.dynamic}
  </Child>
);

const template8 = (
  <Child>
    {item => item}
    {item => item}
  </Child>
);