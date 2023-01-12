const Component = (
  <For each={state.list} fallback={<Loading />}>
    {item => <Show when={state.condition}>{item}</Show>}
  </For>
);
