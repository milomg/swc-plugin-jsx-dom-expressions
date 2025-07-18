import { Show } from "somewhere"

const Child = props => {
  const [s, set] = createSignal();
  return <>
    <div ref={props.ref}>Hello {props.name}</div>
    <div ref={set}>{props.children}</div>
  </>
};

export const template = props => {
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

export const template2 = (
  <Child
    name="Jake"
    dynamic={state.data}
    stale={/*@once*/ state.data}
    handleClick={clickHandler}
    hyphen-ated={state.data}
    ref={el => (e = el)}
  />
);

export const template3 = (
  <Child>
    <div />
    <div />
    <div />
    After
  </Child>
);

const [s, set] = createSignal();
export const template4 = <Child ref={set}>{<div />}</Child>;

export const template5 = <Child dynamic={state.dynamic}>{state.dynamic}</Child>;

// builtIns
export const template6 = (
  <For each={state.list} fallback={<Loading />}>
    {item => <Show when={state.condition}>{item}</Show>}
  </For>
);

export const template7 = (
  <Child>
    <div />
    {state.dynamic}
  </Child>
);

export const template8 = (
  <Child>
    {item => item}
    {item => item}
  </Child>
);

export const template9 = <_garbage>Hi</_garbage>;

export const template10 = (
  <div>
    <Link>new</Link>
    {" | "}
    <Link>comments</Link>
    {" | "}
    <Link>show</Link>
    {" | "}
    <Link>ask</Link>
    {" | "}
    <Link>jobs</Link>
    {" | "}
    <Link>submit</Link>
  </div>
);

export const template11 = (
  <div>
    <Link>new</Link>
    {" | "}
    <Link>comments</Link>
    <Link>show</Link>
    {" | "}
    <Link>ask</Link>
    <Link>jobs</Link>
    {" | "}
    <Link>submit</Link>
  </div>
);

export const template12 = (
  <div>
    {" | "}
    <Link>comments</Link>
    {" | "}
    {" | "}
    {" | "}
    <Link>show</Link>
    {" | "}
  </div>
);

class Template13 {
  render() {
    <Component prop={this.something} onClick={() => this.shouldStay}>
      <Nested prop={this.data}>{this.content}</Nested>
    </Component>;
  }
}

export const Template14 = <Component>{data()}</Component>;

export const Template15 = <Component {...props}/>

export const Template16 = <Component something={something} {...props}/>

export const Template17 = <Pre><span>1</span> <span>2</span> <span>3</span></Pre>

export const Template18 = <Pre>
  <span>1</span>
  <span>2</span>
  <span>3</span>
</Pre>

export const Template19 = <Component {...s.dynamic()} />

export const Template20 = <Component class={prop.red ? "red" : "green"} />

export const template21 = (
  <Component
    {...{
      get [key()]() {
        return props.value;
      }
    }}
  />
);

export const template22 = <Component passObject={{ ...a }} ></Component>

export const template23 = <Component disabled={"t" in test}>{"t" in test && "true"}</Component>

export const template24 = <Component>
  {state.dynamic}
</Component>

export const template25 = <Component>
  <div />
</Component>
