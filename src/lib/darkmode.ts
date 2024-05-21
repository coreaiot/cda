import { writable } from 'svelte/store';
import { appWindow } from '@tauri-apps/api/window';
import { getTheme, Theme } from '@kuyoonjo/tauri-plugin-theme-api';

export const darkmode = writable<'light' | 'dark'>('light');

export async function initDarkmode() {
  let theme = await getTheme();
  console.log('theme', theme);
  if (theme === Theme.Auto) {
    let t = await appWindow.theme() || 'light';
    theme = t === 'light' ? Theme.Light : Theme.Dark;
  }
  darkmode.subscribe(async _theme => {
    document.body.parentElement!.className = _theme;
  });
  darkmode.set(theme);
}

export async function setDarkmode(theme: 'auto' | 'light' | 'dark') {
  if (theme === 'auto') {
    let t = await appWindow.theme() || 'light';
    theme = t === 'light' ? Theme.Light : Theme.Dark;
  }
  darkmode.set(theme);
}