<template>
  <div>
    <main>
      <a-form-model :model="config" layout="inline" :label-col="{span:5}" labelAlign="left">
        <a-form-model-item label="接口网址" :wrapper-col="{span:16}">
          <a-input v-model="config.DOWNLOAD_API" style="width:320px" :disabled="processing" />
        </a-form-model-item>
        <a-form-model-item label="来源网址" :wrapper-col="{span:16}">
          <a-tooltip placement="top">
            <template slot="title">
              <span>部分接口限制了来源地址。</span>
            </template>
            <a-input v-model="config.DOWNLOAD_REFERER" placeholder="如果接口判断来源的话。默认接口地址。" style="width:320px" :disabled="processing" />
          </a-tooltip>
        </a-form-model-item>
        <a-form-model-item label="保存目录" :wrapper-col="{span:16}">
          <a-tooltip placement="top">
            <template slot="title">
              <span>保存目录需要真实存在，程序不会创建文件夹。</span>
            </template>
            <a-input v-model="config.DOWNLOAD_PATH" placeholder="保存目录" style="width:320px" @click="openDirectory" read-only :disabled="processing" />
          </a-tooltip>
        </a-form-model-item>

        <a-form-model-item label="最大出现次数" :label-col="{span:13}" :wrapper-col="{span:11}">
          <a-tooltip placement="top">
            <template slot="title">
              <span>如果获取到同一张图片的次数超过这个。则停止执行。设置为0则不限制。</span>
            </template>
            <a-input-number v-model="config.DOWNLOAD_MAX" style="width:100px" :min="0" :max="10" :disabled="processing" />
          </a-tooltip>
        </a-form-model-item>
        <a-form-model-item label="下载并发数" :label-col="{span:13}" :wrapper-col="{span:11}">
          <a-input-number v-model="config.DOWNLOAD_QUEUE" style="width:90px" placeholder="下载并发数" :min="1" :max="20" :disabled="processing" />
        </a-form-model-item>

        <a-form-model-item label="下载限速" style="position: absolute;top: 8px;" :label-col="{span:13}" :wrapper-col="{span:11}">
          <a-input-number v-model="config.DOWNLOAD_DELAY" style="width:90px" placeholder="单位：秒" :min="0" :disabled="processing" />
        </a-form-model-item>

        <a-form-model-item label="只做记录" style="position: absolute;" :label-col="{span:13}" :wrapper-col="{span:11}">
          <div style="width:90px">
            <a-switch v-model="config.ONLY_RECORD" :disabled="processing" />
          </div>
        </a-form-model-item>

        <a-form-model-item style="width: 120px;position: absolute;top:8px;right:25px;">
          <a-button :type="processing?'danger':'primary'" @click="onSubmit">
            <a-icon :type="processing?'pause':'cloud-download'" />{{processing?'结束':'开始'}}队列
          </a-button>
          <a-button type="dashed" @click="exportRecord" :disabled="list.length===0">
            <a-icon type="export" />导出结果
          </a-button>
        </a-form-model-item>

      </a-form-model>

      <a-table :columns="columns" :data-source="list" :pagination="{pageSize:200}" style="min-height:515px;">
        <span slot="filename" slot-scope="text, record">
          <a @click="openFile(record.filename)">{{record.filename}}</a>
        </span>

        <span slot="url" slot-scope="text, record">
          <a @click="shell.openExternal(record.url)">{{record.url}}</a>
        </span>

        <span slot="size" slot-scope="text, record">
          <span :style="{color:record.repetitions>1?'red':'green'}">{{record.repetitions}}</span>
        </span>
        <span slot="size" slot-scope="text, record">
          {{prettyBytes(record.transferred)}}/{{prettyBytes(record.size)}}
        </span>
        <span slot="progress" slot-scope="text, record">
          <a-progress :percent="parseInt(record.percent*100)" :status="record.status" size="small" />
        </span>
      </a-table>
    </main>
    <a-layout-footer>
      ©{{new Date().getFullYear()}} <a @click="shell.openExternal('https://qiangmou.ren')">强某人</a> |
      <a @click="shell.openExternal('https://github.com/qiangmouren/images-thief')">https://github.com/qiangmouren/images-thief</a>
    </a-layout-footer>

  </div>
</template>
<script>
const electron = require('electron')
const remote = electron.remote;
const ipcRenderer = electron.ipcRenderer;
const path = require('path');
const prettyBytes = require('pretty-bytes');

export default {
  data () {
    return {
      shell: remote.shell,
      prettyBytes,
      list: [],
      columns: [
        {
          title: '任务ID',
          ellipsis: true,
          width: 120,
          dataIndex: 'uuid'
        },
        {
          title: '网址',
          dataIndex: 'url',
          ellipsis: true,
          width: 300,
          scopedSlots: { customRender: 'url' },
        },
        {
          title: '文件名',
          ellipsis: true,
          dataIndex: 'filename',
          width: 150,
          scopedSlots: { customRender: 'filename' },
        },
        {
          title: '大小',
          dataIndex: 'size',
          width: 150,
          scopedSlots: { customRender: 'size' },
        },
        {
          title: '出现次数',
          dataIndex: 'repetitions',
          width: 100,
          scopedSlots: { customRender: 'repetitions' },
        },
        {
          title: '下载进度',
          key: 'progress',
          scopedSlots: { customRender: 'progress' },
        },
      ],
      processing: false,
      config: {
        DOWNLOAD_API: 'https://api.vvhan.com/api/acgimg',
        DOWNLOAD_REFERER: '',
        DOWNLOAD_MAX: 3,
        DOWNLOAD_PATH: '',
        DOWNLOAD_QUEUE: 1,
        DOWNLOAD_DELAY: 0,
        ONLY_RECORD: false
      },
    };
  },
  mounted () {
    ipcRenderer.send('get-config')
    ipcRenderer.on('got-config', (event, config) => {
      this.config.DOWNLOAD_PATH = path.join(config.desktopDir, 'images')
    });
    ipcRenderer.on('notify', (event, opt) => {
      this.$message[opt.type](opt.message)
    });
    ipcRenderer.on('queue-result', (event, data) => {
      this.processing = !!data.processing;
      this.list = data.list || [];
    });
  },
  methods: {
    async openDirectory () {
      const { filePaths: [path] } = await remote.dialog.showOpenDialog({ properties: ['openDirectory'] });
      if (!path) {
        this.$message.error('请选择保存目录。')
        return;
      }
      this.config.DOWNLOAD_PATH = path;
    },
    async openFile (filename) {
      const file = path.join(this.config.DOWNLOAD_PATH, filename);
      remote.shell.showItemInFolder(file);
    },
    async exportRecord () {
      ipcRenderer.send('export-record');
    },
    async onSubmit () {
      if (!this.processing) {
        try { new URL(this.config.DOWNLOAD_API) } catch (error) {
          return this.$message.error('请输入正确的接口地址。')
        }
        try { this.config.DOWNLOAD_REFERER && new URL(this.config.DOWNLOAD_REFERER) } catch (error) {
          return this.$message.error('请输入正确的来源地址。')
        }
        if (!this.config.DOWNLOAD_PATH) {
          return this.$message.error('请选择保存目录。')
        }
        if (this.config.DOWNLOAD_QUEUE < 1) {
          return this.$message.error('下载并发数最低为1。')
        }
        ipcRenderer.send('start-queue', this.config);
      } else {
        ipcRenderer.send('stop-queue');
      }
    },
  },
};
</script>
<style>
body,
html {
  width: 100%;
  height: 100%;
}
main {
  width: 100%;
  height: 100%;
  position: relative;
  padding: 8px;
}
footer {
  width: 100%;
  /* position: absolute; */
  bottom: 0;
  text-align: center;
}
</style>