/**
 * Entry point for the Svelte application.
 * This file mounts the main App.svelte component onto the DOM.
 */
import App from './App.svelte';
import { mount } from 'svelte';

// Mount the root component to the <div id="app"> element in index.html
const app = mount(App, { target: document.getElementById('app') });

export default app;
