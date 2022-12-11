/**
 * @file: main.js
 * @description: main.js
 * @package: images-thief
 * @create: 2022-12-11 10:49:27
 * @author: qiangmouren (2962051004@qq.com)
 * -----
 * @last-modified: 2022-12-12 01:12:46
 * -----
 */
import { createApp } from 'vue';
import './style.css';
import 'element-plus/dist/index.css';

import ElementPlus from 'element-plus';
import App from './App.vue';

const app = createApp(App);

app.use(ElementPlus);
app.mount('#app');
