const template = (
  <div id="main">
    <style>{"div { color: red; }"}</style>
    <h1>Welcome</h1>
    <label for={"entry"}>Edit:</label>
    <input id="entry" type="text" />
    {/* Comment Node */}
  </div>
);

const template2 = (
  <div>
    <span>
      <a></a>
    </span>
    <span />
  </div>
);

const template3 = (
  <div>
    <div>
      <table>
        <tbody></tbody>
      </table>
    </div>
    <div></div>
  </div>
);

const template4 = (
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

const template5 = <>Hello</>
const template6 = <>{"Hello"}</>
const template7 = <>{props.id}</>
const template8 = <>{"1"}{"2"}</>
const template9 = <>{"1"}{props.id}</>
const template10 = <>{1}</>
const template11 = <>{`Hello ${props.name}`}</>
let id = 123;
const template12 = <>{id}</>
const signal = () => 1;
const template13 = <>{signal()}</>
