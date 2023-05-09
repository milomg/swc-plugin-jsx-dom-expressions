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