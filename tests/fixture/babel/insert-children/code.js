const children = <div />;
const dynamic = {
  children
};
export const template = <Module children={children} />;
export const template2 = <module children={children} />;
export const template3 = <module children={children}>Hello</module>;
export const template4 = (
  <module children={children}>
    <Hello />
  </module>
);
export const template5 = <module children={dynamic.children} />;
export const template6 = <Module children={dynamic.children} />;
export const template7 = <module {...dynamic} />;
export const template8 = <module {...dynamic}>Hello</module>;
export const template9 = <module {...dynamic}>{dynamic.children}</module>;
export const template10 = <Module {...dynamic}>Hello</Module>;
export const template11 = <module children={/*@once*/ state.children} />;
export const template12 = <Module children={/*@once*/ state.children} />;
export const template13 = <module>{...children}</module>;
export const template14 = <Module>{...children}</Module>;
export const template15 = <module>{...dynamic.children}</module>;
export const template16 = <Module>{...dynamic.children}</Module>;
export const template18 = <module>Hi {...children}</module>;
export const template19 = <Module>Hi {...children}</Module>;
export const template20 = <module>{children()}</module>;
export const template21 = <Module>{children()}</Module>;
export const template22 = <module>{state.children()}</module>;
export const template23 = <Module>{state.children()}</Module>;
export const template24 = <module {...dynamic}>Hi{dynamic.children}</module>;

const tiles = [];
tiles.push(<div>Test 1</div>);
export const template25 = <div>{tiles}</div>;

export const comma = <div>{expression(), "static"}</div>
