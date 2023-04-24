const f = () => "a"

const a = () => {
    let b = f();
    return <div class={f()}>
        <p title={b + "foo"}>hi</p>
    </div>
};
