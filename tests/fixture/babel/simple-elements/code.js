export const template = (
  <div id="main">
    <style>{"div { color: red; }"}</style>
    <h1>Welcome</h1>
    <label for={"entry"}>Edit:</label>
    <input id="entry" type="text" />
    {/* Comment Node */}
  </div>
);

export const template2 = (
  <div>
    <span>
      <a></a>
    </span>
    <span />
  </div>
);

export const template3 = (
  <div>
    <div>
      <table>
        <tbody></tbody>
      </table>
    </div>
    <div></div>
  </div>
);

export const template4 = (
  <div>
    <div>
      <footer>
        <div />
      </footer>
    </div>
    <div>
      <button>
        <span>{0}</span>
      </button>
    </div>
  </div>
);

export const template5 = <>Hello</>
export const template6 = <>{"Hello"}</>
export const template7 = <>{props.id}</>
export const template8 = <>{"1"}{"2"}</>
export const template9 = <>{"1"}{props.id}</>
export const template10 = <>{1}</>
export const template11 = <>{`Hello ${props.name}`}</>
let id = 123;
export const template12 = <>{id}</>
const signal = () => 1;
export const template13 = <>{signal()}</>
