import { invoke } from "@tauri-apps/api/core";
import { createApp } from "vue";
import App from "./App.vue";
import "./styles/app.css";

createApp(App).mount("#app");

void invoke("show_main_window").catch((error) => {
  console.error("[main] failed to reveal window after mount", error);
});
