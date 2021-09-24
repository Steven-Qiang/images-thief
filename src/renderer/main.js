import Vue from 'vue'

import App from './App'

import Ant from 'ant-design-vue';


import 'ant-design-vue/dist/antd.css';

Vue.config.productionTip = false
Vue.use(Ant)

/* eslint-disable no-new */
new Vue({
  components: { App },
  template: '<App/>'
}).$mount('#app')
