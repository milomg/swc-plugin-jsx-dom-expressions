import { Show } from "somewhere"

const Child = props => {
  const [s, set] = createSignal();
  return <>
    <div ref={props.ref}>Hello {props.name}</div>
    <div ref={set}>{props.children}</div>
  </>
};