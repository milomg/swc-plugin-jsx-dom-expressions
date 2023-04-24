const a = (size) => {
    const s = {"font-size": `${size}px`};
    return <div style={{
        width: size,
        height: size * 2,
    }}><p style={s}>hi</p></div>
};
