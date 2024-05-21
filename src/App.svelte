<script lang="ts">
  import { appWindow, WebviewWindow } from "@tauri-apps/api/window";
  import { onDestroy, onMount } from "svelte";
  import { initDarkmode, setDarkmode } from "./lib/darkmode";
  import { listen } from "@tauri-apps/api/event";
  import Udp from "./lib/UDP.svelte";
  import { Buffer } from "buffer";

  (window as any).Buffer = Buffer;

  function test() {
    console.log(appWindow);
    // loading embedded asset:
    const webview = new WebviewWindow(new Date().getTime().toString(), {
      url: "/?udp",
    });

    webview.once("tauri://created", function () {
      // webview window successfully created
    });
    webview.once("tauri://error", function (e) {
      // an error happened creating the webview window
    });

    webview.setTitle("xxx");
    webview.show();
  }

  let hash = "";

  let unsubscribe = () => {};
  onMount(async () => {
    hash = window.location.hash || "#udp";
    appWindow.setTitle("Coreaiot Debug Assistant " + hash);
    await initDarkmode();

    const unlisten = await listen<string>("theme-change", (event) => {
      setDarkmode(event.payload as any);
    });

    unsubscribe = () => {
      unlisten();
    };
  });

  onDestroy(() => {
    unsubscribe();
  });
</script>

<main class="container">
  {#if hash === "#udp"}
    <Udp />
  {/if}
</main>
