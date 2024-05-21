<script lang="ts">
  import { dialog, invoke } from "@tauri-apps/api";
  import AutoFill from "./components/AutoFill.svelte";
  import moment from "moment";
  import { onDestroy, onMount } from "svelte";
  import storage from "./storage";
  import { Buffer } from "buffer";
  import { getCurrent } from "@tauri-apps/api/window";
  import Select from "./components/Select.svelte";
  import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
  import { downloadDir } from "@tauri-apps/api/path";
  import { listen } from "@tauri-apps/api/event";

  let windowId: string = "";

  const defaultLocalItems = ["0.0.0.0:8080", "127.0.0.1:8080"];
  const k_exLocalItems = "exLocalItems";
  let exLocalItems: string[] = [];
  let localItems: string[] = [];
  let local = "";
  let bondAt = "";

  const defaultRemoteItems = ["255.255.255.255:8256", "192.168.1.255:8256"];
  const k_exRemoteItems = "exRemoteItems";
  let exRemoteItems: string[] = [];
  let remoteItems: string[] = [];
  let remote = "";

  const defaultFuncItems: {
    name: string;
    input: boolean;
    args: {
      name: string;
      defaultValue: string;
    }[];
    type: "string" | "binary";
    fn?: (...args: string[]) => string;
  }[] = [
    {
      name: "string",
      input: true,
      args: [],
      type: "string",
    },
    {
      name: "binary",
      input: true,
      args: [],
      type: "binary",
    },
  ];
  const k_exFuncItems = "exFuncItems";
  let funcItems = [...defaultFuncItems];
  let exFuncItems: { name: string; js: string }[] = [];
  let func = funcItems[0];
  let funcArgs = func.args.map((x) => ({ k: x.name, v: x.defaultValue }));
  function onFuncChange() {
    funcArgs = func.args.map((x) => ({ k: x.name, v: x.defaultValue }));
    refreshInput();
  }
  function refreshInput() {
    if (func.fn) {
      input = func.fn(...funcArgs.map((x) => x.v));
    }
  }
  async function parseFuncItemJs(str: string) {
    var b64 = "data:text/javascript," + str;
    const m: (typeof defaultFuncItems)[number] = await import(b64);
    console.log(m);
    const item: (typeof defaultFuncItems)[number] = {
      name: m.name,
      args: m.args,
      input: false,
      fn: m.fn,
      type:
        typeof m.fn!(...m.args.map((x) => x.defaultValue)) === "string"
          ? "string"
          : "binary",
    };
    return item;
  }
  async function importFuncItem() {
    const p = await dialog.open({
      filters: [
        {
          name: "Javascript",
          extensions: ["js"],
        },
      ],
    });
    if (p) {
      const js = await readTextFile(p as string);
      const m = await parseFuncItemJs(js);
      let i = funcItems.findIndex((x) => x.name === m.name);
      if (~i) funcItems[i] = m;
      else funcItems.push(m);
      funcItems = funcItems;
      i = exFuncItems.findIndex((x) => x.name === m.name);
      if (~i) exFuncItems[i] = { name: m.name, js };
      else exFuncItems.push({ name: m.name, js });
      storage.set(k_exFuncItems, exFuncItems);
      func = m;
      onFuncChange();
    }
  }
  async function exportFuncItem() {
    try {
      const filePath = (await invoke("open_save_dialog", {
        dir: await downloadDir(),
        defaultFileName: func.name.replaceAll(" ", "_") + ".js",
      })) as string;
      console.log("filePath", filePath);
      console.log(func, exFuncItems);
      const ex = exFuncItems.find((x) => x.name === func.name);
      if (ex) {
        console.log("writeTextFile", filePath);
        await writeTextFile(filePath, ex.js);
      }
    } catch (e) {
      console.error(e);
    }
  }
  async function deleteFuncItem() {
    exFuncItems = storage.get<typeof exFuncItems>(k_exFuncItems) || [];
    let i = exFuncItems.findIndex((x) => x.name === func.name);
    if (~i) {
      exFuncItems.splice(i, 1);
      storage.set(k_exFuncItems, exFuncItems);
    }
    i = funcItems.indexOf(func);
    if (~i) {
      funcItems.splice(i, 1);
      func = funcItems[0];
    }
  }
  async function loadExFuncItems() {
    exFuncItems = storage.get<typeof exFuncItems>(k_exFuncItems) || [];
    for (const item of exFuncItems) {
      const m = await parseFuncItemJs(item.js);
      funcItems.push(m);
      funcItems = funcItems;
    }
  }

  const defaultParserItems: {
    name: string;
    custom?: boolean;
    fn: (buf: number[]) => string;
  }[] = [
    {
      name: "string",
      fn: (buf) => JSON.stringify(Buffer.from(buf).toString()),
    },
    {
      name: "binary",
      fn: (buf) => buf.map((x) => x.toString(16).padStart(2, "0")).join(" "),
    },
  ];
  const k_exParserItems = "exParserItems";
  let parserItems = [...defaultParserItems];
  let exParserItems: { name: string; js: string }[] = [];
  let parser = parserItems[0];
  async function parseParserItemJs(str: string) {
    var b64 = "data:text/javascript," + str;
    const m: (typeof defaultParserItems)[number] = await import(b64);
    const item: typeof m = {
      name: m.name,
      fn: m.fn,
      custom: true,
    };
    return item;
  }
  async function importParserItem() {
    const p = await dialog.open({
      filters: [
        {
          name: "Javascript",
          extensions: ["js"],
        },
      ],
    });
    if (p) {
      const js = await readTextFile(p as string);
      const m = await parseParserItemJs(js);
      let i = parserItems.findIndex((x) => x.name === m.name);
      if (~i) parserItems[i] = m;
      else parserItems.push(m);
      parserItems = parserItems;
      i = exParserItems.findIndex((x) => x.name === m.name);
      if (~i) exParserItems[i] = { name: m.name, js };
      else exParserItems.push({ name: m.name, js });
      storage.set(k_exParserItems, exParserItems);
      parser = m;
    }
  }
  async function exportParserItem() {
    try {
      const filePath = (await invoke("open_save_dialog", {
        dir: await downloadDir(),
        defaultFileName: parser.name.replaceAll(" ", "_") + ".js",
      })) as string;
      const ex = exParserItems.find((x) => x.name === parser.name);
      if (ex) {
        console.log("writeTextFile", filePath);
        await writeTextFile(filePath, ex.js);
      }
    } catch (e) {
      console.error(e);
    }
  }
  async function deleteParserItem() {
    exParserItems = storage.get<typeof exParserItems>(k_exParserItems) || [];
    let i = exParserItems.findIndex((x) => x.name === parser.name);
    if (~i) {
      exParserItems.splice(i, 1);
      storage.set(k_exParserItems, exParserItems);
    }
    i = parserItems.indexOf(parser);
    if (~i) {
      parserItems.splice(i, 1);
      parser = parserItems[0];
    }
  }
  async function loadExParserItems() {
    exParserItems = storage.get<typeof exFuncItems>(k_exParserItems) || [];
    for (const item of exParserItems) {
      const m = await parseParserItemJs(item.js);
      parserItems.push(m);
      parserItems = parserItems;
    }
  }

  let output = "";
  function addOutput(str: string) {
    output = `[${moment().format("YYYY-MM-DD HH:mm:ss.SSS")}] ${str}\n${output}`;
  }
  function clearOutput() {
    output = "";
  }
  async function bind() {
    const ok = await invoke("udp_bind", { id: windowId, bindAt: local });
    if (ok) addOutput(`UDP bond at ${local}`);
    else addOutput(`UDP failed to bind at ${local}`);
    if (!localItems.includes(local)) {
      exLocalItems.unshift(local);
      storage.set(k_exLocalItems, exLocalItems);
      localItems.unshift(local);
      localItems = localItems;
    }
    exLocalItems = exLocalItems;
    local = local;
    bondAt = local;
  }
  async function unbind() {
    const ok = await invoke("udp_unbind", { id: windowId });
    if (ok) {
      addOutput(`UDP unbond from ${local}`);
      bondAt = "";
    } else addOutput(`UDP failed to unbind from ${local}`);
  }

  let input = "";
  async function send() {
    const message =
      func.type === "binary"
        ? Array.from(
            Buffer.from(
              input
                .split(/\s+/)
                .map((x) => (x.length % 2 ? "0" + x : x))
                .join(""),
              "hex",
            ),
          )
        : Array.from(Buffer.from(input));
    const ok = await invoke("udp_send", {
      id: windowId,
      target: remote,
      message,
    });
    const data = func.type === 'binary' ? message.map(x => x.toString(16).padStart(2, '0')).join(' ') : input;
    if (ok) addOutput(`→ [${remote}] ${data} → OK`);
    else addOutput(`→ [${remote}] ${data} → FAIL`);
    if (!remoteItems.includes(remote)) {
      exRemoteItems.unshift(remote);
      storage.set(k_exRemoteItems, exRemoteItems);
      remoteItems.unshift(remote);
      remoteItems = remoteItems;
    }
    exRemoteItems = exRemoteItems;
    remote = remote;
  }
  function clearLocalItem() {
    local = "";
  }
  function deleteLocalItem() {
    exLocalItems = storage.get(k_exLocalItems) || [];
    const i = exLocalItems.indexOf(local);
    if (~i) {
      exLocalItems.splice(i, 1);
      storage.set(k_exLocalItems, exLocalItems);
    }
    localItems = [...exLocalItems, ...defaultLocalItems];
    local = "";
  }
  function clearRemoteItem() {
    remote = "";
  }
  function deleteRemoteItem() {
    exRemoteItems = storage.get(k_exRemoteItems) || [];
    const i = exRemoteItems.indexOf(remote);
    if (~i) {
      exRemoteItems.splice(i, 1);
      storage.set(k_exRemoteItems, exRemoteItems);
    }
    remoteItems = [...exRemoteItems, ...defaultRemoteItems];
    remote = "";
  }

  let unlisten = () => {};
  onMount(async () => {
    windowId = getCurrent().label;
    exLocalItems = storage.get(k_exLocalItems) || [];
    localItems = [...exLocalItems, ...defaultLocalItems];
    local = defaultLocalItems[0];
    exRemoteItems = storage.get(k_exRemoteItems) || [];
    remoteItems = [...exRemoteItems, ...defaultRemoteItems];
    remote = defaultRemoteItems[0];
    await loadExFuncItems();
    await loadExParserItems();
    unlisten = await listen<{
      addr: string;
      data: number[];
    }>("udp", (e) => {
      const data = parser.fn(e.payload.data);
      addOutput(`← [${e.payload.addr}] ${data}`);
    });
  });

  onDestroy(() => {
    unlisten();
  });
</script>

<main>
  <div style="display: flex; margin-top: 12px; align-items: center;">
    <!-- svelte-ignore a11y-label-has-associated-control -->
    <label>Local</label>
    <AutoFill
      placeholder="e.g. 127.0.0.1:8080"
      bind:value={local}
      items={localItems}
      disabled={!!bondAt}
    />
    {#if bondAt}
      <button class="btn error" on:click={unbind}>Unbind</button>
    {:else}
      <button class="btn primary" on:click={bind}>Bind</button>
      <button class="btn primary" on:click={clearLocalItem}>Clear</button>
      {#if exLocalItems.includes(local)}
        <button class="btn error" on:click={deleteLocalItem}>Delete</button>
      {/if}
    {/if}
    <!-- svelte-ignore a11y-label-has-associated-control -->
    <label style="margin-left: 16px">Remote</label>
    <AutoFill
      placeholder="e.g. 192.168.1.102:8256"
      bind:value={remote}
      items={remoteItems}
    />
    <button class="btn primary" on:click={send} disabled={!bondAt}>Send</button>
    <button class="btn primary" on:click={clearRemoteItem}>Clear</button>
    {#if exLocalItems.includes(local)}
      <button class="btn error" on:click={deleteRemoteItem}>Delete</button>
    {/if}
  </div>

  <hr />

  <div style="display: flex; align-items: center;">
    <!-- svelte-ignore a11y-label-has-associated-control -->
    <label>Function</label>
    <Select
      bind:value={func}
      items={funcItems}
      key="name"
      on:change={onFuncChange}
    />
    <button class="btn primary" on:click={importFuncItem}>Import</button>
    {#if !func.input}
      <button class="btn primary" on:click={exportFuncItem}>Export</button>
      <button class="btn error" on:click={deleteFuncItem}>Delete</button>
    {/if}
  </div>
  {#if funcArgs.length}
    <div style="display: flex; align-items: center;margin: 8px 0 0;">
      {#each funcArgs as arg}
        <!-- svelte-ignore a11y-label-has-associated-control -->
        <label>{arg.k}</label>
        <input
          style="margin-right: 12px"
          type="text"
          bind:value={arg.v}
          on:input={refreshInput}
        />
      {/each}
    </div>
  {/if}
  <textarea
    style="margin: 8px 0 0; flex-grow: 1;"
    bind:value={input}
    disabled={!func.input}
  ></textarea>
  <hr />
  <div style="display: flex; align-items: center;">
    <!-- svelte-ignore a11y-label-has-associated-control -->
    <label>Parser</label>
    <Select bind:value={parser} items={parserItems} key="name" />
    <button class="btn primary" on:click={importParserItem}>Import</button>
    {#if parser.custom}
      <button class="btn primary" on:click={exportParserItem}>Export</button>
      <button class="btn error" on:click={deleteParserItem}>Delete</button>
    {/if}
    <div style="flex-grow: 1;"></div>
    <button class="btn primary" on:click={clearOutput}>Clear</button>
  </div>
  <textarea
    style="margin: 8px 0 12px; flex-grow: 1"
    disabled
    bind:value={output}
  ></textarea>
</main>

<style lang="scss">
  main {
    padding: 0 12px;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: stretch;
  }

  label {
    margin: 0 8px 0 0;
  }

  .btn {
    margin-left: 8px;
  }

  textarea {
    resize: none;
    padding: 10px;
    line-height: 15px;
  }
</style>
