import 'highlight.js/styles/atom-one-dark.css';
import {invoke} from "@tauri-apps/api";
import highlight from 'highlight.js/lib/core';
import lua from 'highlight.js/lib/languages/lua';
import {listen} from "@tauri-apps/api/event";
import {appWindow} from "@tauri-apps/api/window";

// Reduce bundle size by only including syntax highlighting support for Lua
highlight.registerLanguage('lua', lua);

/**
 * Updates the window with the decrypted content from the backend. `filename` will be used to update the title of
 * the window, while `content` will be placed within a <pre><code> block.
 *
 * @param {string | undefined} filename
 * @param {string} content
 */
const updateApp = ({ filename, content }) => {
    appWindow.setTitle(`${filename ? `${filename} - ` : ''}Telltale Resdesc Viewer`);
    document.querySelector('#app').innerHTML = `<pre><code>${content}</code></pre>`;
    highlight.highlightAll();
}

// On application load, get the contents of the initial path given to the application.
invoke('get_initial_contents').then(data => {
    updateApp(data ?? { content: 'No file loaded.' });
});

// When new content updates arrive (via clicking file > open in the menu), update the app with the new contents.
listen('content', data => updateApp(data.payload));