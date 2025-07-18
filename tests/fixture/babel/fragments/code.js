export const multiStatic = (
  <>
    <div>First</div>
    <div>Last</div>
  </>
);

export const multiExpression = (
  <>
    <div>First</div>
    {inserted}
    <div>Last</div>
    After
  </>
);

export const multiDynamic = (
  <>
    <div id={state.first}>First</div>
    {state.inserted}
    <div id={state.last}>Last</div>
    After
  </>
);

export const singleExpression = <>{inserted}</>;

export const singleDynamic = <>{inserted()}</>;

export const firstStatic = (
  <>
    {inserted}
    <div />
  </>
);

export const firstDynamic = (
  <>
    {inserted()}
    <div />
  </>
);

export const firstComponent = (
  <>
    <Component />
    <div />
  </>
);

export const lastStatic = (
  <>
    <div />
    {inserted}
  </>
);

export const lastDynamic = (
  <>
    <div />
    {inserted()}
  </>
);

export const lastComponent = (
  <>
    <div />
    <Component />
  </>
);

export const spaces = <><span>1</span> <span>2</span> <span>3</span></>
export const multiLineTrailing = <>
  <span>1</span>
  <span>2</span>
  <span>3</span>
</>
