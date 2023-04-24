const A = () => (
    <div>A</div>
);

const B = (b) => (
    <div>
        <A/>
        <p class={b}>b</p>
        <A/>
        <p className={b}>c</p>
    </div>
);
