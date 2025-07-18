export const trailing = <span>Hello </span>;
export const leading = <span> John</span>;

/* prettier-ignore */
export const extraSpaces = <span>Hello   John</span>;

export const trailingExpr = <span>Hello {name}</span>;
export const leadingExpr = <span>{greeting} John</span>;

/* prettier-ignore */
export const multiExpr = <span>{greeting} {name}</span>;

/* prettier-ignore */
export const multiExprSpaced = <span> {greeting} {name} </span>;

/* prettier-ignore */
export const multiExprTogether = <span> {greeting}{name} </span>;

/* prettier-ignore */
export const multiLine = <span>

  Hello

</span>

/* prettier-ignore */
export const multiLineTrailingSpace = <span>
  Hello
  John
</span>

/* prettier-ignore */
export const multiLineNoTrailingSpace = <span>
  Hello
  John
</span>

/* prettier-ignore */
export const escape = <span>
  &nbsp;&lt;Hi&gt;&nbsp;
</span>

/* prettier-ignore */
export const escape2 = <Comp>
  &nbsp;&lt;Hi&gt;&nbsp;
</Comp>

/* prettier-ignore */
export const escape3 = <>
  &nbsp;&lt;Hi&gt;&nbsp;
</>

export const injection = <span>Hi{"<script>alert();</script>"}</span>

let value = "World";
export const evaluated = <span>Hello {value + "!"}</span>

let number = 4 + 5;
export const evaluatedNonString = <span>4 + 5 = {number}</span>

export const newLineLiteral = <div>{s}{"\n"}d</div>

export const trailingSpace = <div>
  {expr}
</div>

export const trailingSpaceComp = <Comp>
  {expr}
</Comp>

export const trailingSpaceFrag = <>
  {expr}
</>

export const leadingSpaceElement = <span> {expr}</span>

export const leadingSpaceComponent = <Div> {expr}</Div>

export const leadingSpaceFragment = <> {expr}</>

export const trailingSpaceElement = <span>{expr} </span>

export const trailingSpaceComponent = <Div>{expr} </Div>

export const trailingSpaceFragment = <>{expr} </>

export const escapeAttribute = <div normal="Search&hellip;" title={"Search&hellip;"} />

export const escapeCompAttribute = <Div normal="Search&hellip;" title={"Search&hellip;"} />

export const lastElementExpression = <div><div></div>{expr()}</div>;
