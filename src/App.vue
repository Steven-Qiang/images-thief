<template>
  <h2 style="margin-left: 50px">Images Thief 图片小偷</h2>
  <h5 style="margin-left: 63px">批量下载随机图片接口的所有图片</h5>
  <el-form ref="formRef" :model="form" :rules="rules" label-width="130px" :size="formSize" status-icon>
    <el-form-item label="接口地址" prop="apiUrl">
      <el-input v-model="form.apiUrl" style="max-width: 500px" />
    </el-form-item>
    <el-form-item label="保存目录" prop="outputDir">
      <el-input
        v-model="form.outputDir"
        @click="tapOutputDir"
        :readonly="true"
        style="max-width: 500px"
        @mousedown="(e) => e.preventDefault()"
      />
    </el-form-item>
    <el-form-item label="更多选项">
      <el-checkbox-group v-model="form.options">
        <el-tooltip class="box-item" effect="dark" content="部分接口可能限制了来源地址。" placement="top">
          <el-checkbox label="setRefererUrl">来源地址</el-checkbox>
        </el-tooltip>
        <el-tooltip class="box-item" effect="dark" content="设置请求和下载的并发数限制。" placement="top">
          <el-checkbox
            label="setConcurrency"
            @change="
              () => {
                if (form.options.includes('setConcurrency')) {
                  form.rawOptions.concurrency = 5;
                }
              }
            "
            >设置并发数</el-checkbox
          >
        </el-tooltip>
        <el-tooltip class="box-item" effect="dark" placement="top">
          <template #content>
            运行一定时间后可能会出现耗尽的现象。<br />
            此为同一图片出现的次数的阈值设置。<br />
            一旦超过将停止进程进行。设置为0则无视。
          </template>
          <el-checkbox
            label="setMaxDuplicate"
            @change="
              () => {
                if (form.options.includes('setMaxDuplicate')) {
                  form.rawOptions.maxDuplicate = 3;
                }
              }
            "
            >设置出现最大阈值</el-checkbox
          >
        </el-tooltip>
        <el-tooltip class="box-item" effect="dark" content="只保存下载链接，便于导出。不下载图片。" placement="top">
          <el-checkbox
            label="setNoDownload"
            @change="
              () => {
                form.rawOptions.noDownload = form.options.includes('setNoDownload');
              }
            "
            >不下载图片</el-checkbox
          >
        </el-tooltip>
      </el-checkbox-group>
    </el-form-item>
    <div style="display: flex">
      <el-form-item label="并发数" v-if="isSetConcurrency" prop="rawOptions.concurrency">
        <el-input-number v-model="form.rawOptions.concurrency" style="width: 120px" :min="1" />
      </el-form-item>
      <el-form-item label="设置阈值" v-if="isSetMaxDuplicate" prop="rawOptions.maxDuplicate">
        <el-input-number v-model="form.rawOptions.maxDuplicate" style="width: 120px" :min="0" />
      </el-form-item>
    </div>
    <el-form-item label="来源地址" v-if="isSetRefererUrl" prop="rawOptions.refererUrl">
      <el-input v-model="form.rawOptions.refererUrl" style="max-width: 500px" />
    </el-form-item>
    <el-form-item>
      <el-button :type="running ? 'danger' : 'primary'" @click="tapButton(formRef)">{{ btnText }}</el-button>
      <el-button type="warning" v-if="tableData.length" :disabled="running" @click="tapReset">重置列表</el-button>
      <el-button type="info" v-if="tableData.length" :disabled="running" @click="tapExportCsv">导出列表</el-button>
    </el-form-item>
  </el-form>
  <div class="table">
    <el-table
      ref="tableRef"
      row-key="url"
      :data="tableData"
      border
      :style="{ width: '90%', margin: 'auto' }"
      :default-sort="{ prop: 'duplicate', order: 'ascending' }"
    >
      <el-table-column prop="url" label="图片地址" sortable />
      <el-table-column prop="filename" label="文件名" sortable>
        <template #default="scope">
          <a href="#" @click="openInExplorer(scope.row.filename)">{{ scope.row.filename }}</a>
        </template>
      </el-table-column>
      <el-table-column prop="size" label="文件大小" width="180" v-if="!form.rawOptions.noDownload" sortable>
        <template #default="scope">
          <span>{{ prettyBytes(scope.row.size) }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="duplicate" label="出现次数" width="180" sortable />
      <el-table-column fixed="right" prop="progress" label="进度" width="180" v-if="!form.rawOptions.noDownload">
        <template #default="scope">
          <el-progress :percentage="scope.row.progress" :status="scope.row.status" />
        </template>
      </el-table-column>
    </el-table>
  </div>
  <div class="footer">
    @{{ new Date().getFullYear() }}&nbsp;
    <a @click="shell.open('https://qiangmou.ren')" target="_blank" rel="noopener noreferrer">强某人</a>&nbsp;
    <a @click="shell.open('https://github.com/qiangmouren/images-thief')" target="_blank" rel="noopener noreferrer"
      >https://github.com/qiangmouren/images-thief</a
    >
  </div>
</template>

<script lang="ts" setup>
import { reactive, ref, watch, onMounted, computed } from 'vue';
import { ElMessageBox, FormInstance, FormRules } from 'element-plus';
import { ElTable } from 'element-plus';

import prettyBytes from 'pretty-bytes';
import Queue from 'p-queue';

import * as fs from '@tauri-apps/api/fs';
import * as path from '@tauri-apps/api/path';
import * as dialog from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import * as shell from '@tauri-apps/api/shell';

interface Row {
  url: string;
  size: number;
  duplicate: number;
  filename: string;
  progress: number;
  status: 'exception' | 'success' | '';
}

const tableRef = ref<InstanceType<typeof ElTable>>();
const tableData = ref<Row[]>([]);

let btnText = ref('开始');
let running = ref<boolean>(false);

const formSize = ref('default');
const formRef = ref<FormInstance>();
const form = reactive({
  apiUrl: '', //'https://api.btstu.cn/sjbz/api.php?lx=dongman',
  outputDir: '',
  options: ['setConcurrency', 'setRefererUrl'] as string[],
  rawOptions: {
    refererUrl: '',
    maxDuplicate: 3,
    noDownload: false,
    concurrency: 5,
  },
});

const rules = reactive<FormRules>({
  apiUrl: {
    required: true,
    message: '请输入正确的URL',
    trigger: 'blur',
    type: 'url',
  },
  outputDir: {
    required: true,
    message: '请选择保存目录',
    trigger: 'blur',
  },
  'rawOptions.refererUrl': {
    required: true,
    message: '请输入正确的来源URL',
    trigger: 'blur',
    type: 'url',
  },
  'rawOptions.maxDuplicate': {
    required: true,
    message: '请输入正确的阈值',
    trigger: 'blur',
    type: 'number',
  },
  'rawOptions.concurrency': {
    required: true,
    message: '请输入正确的并发数',
    trigger: 'blur',
    type: 'number',
  },
});

onMounted(async () => {
  form.outputDir = await path.join(await path.desktopDir(), 'download');
  form.rawOptions.refererUrl = form.apiUrl;
});

watch(
  () => form.apiUrl,
  (newVal, oldVal) => {
    if (newVal === oldVal) return;
    form.rawOptions.refererUrl = newVal;
  }
);

const isSetRefererUrl = computed(() => form.options.includes('setRefererUrl'));
const isSetMaxDuplicate = computed(() => form.options.includes('setMaxDuplicate'));
const isSetNoDownload = computed(() => form.options.includes('setNoDownload'));
const isSetConcurrency = computed(() => form.options.includes('setConcurrency'));

let queue: Queue | null;
const stop = async () => {
  btnText.value = '停止中...';
  if (queue) {
    queue.pause();
    queue.clear();
    queue = null;
  }
  running.value = false;
  btnText.value = '开始';
};
const start = async () => {
  running.value = true;
  btnText.value = '停止';
  const concurrency = parseInt(form.rawOptions.concurrency.toString());
  queue = new Queue({
    concurrency,
    interval: 100,
  });
  try {
    const t = (await invoke('fetch', {
      apiUrl: form.apiUrl,
      refUrl: form.rawOptions.refererUrl,
    })) as string;
    if (!t) {
      throw null;
    }
  } catch {
    ElMessageBox.alert('该链接似乎不存在跳转，请检查URL是否正确', '提示');
    stop();
    return;
  }

  await fs.createDir(form.outputDir, { recursive: true }).catch(console.log);

  while (running.value) {
    queue.add(async () => {
      const imgUrl = (await invoke('fetch', {
        apiUrl: form.apiUrl,
        refUrl: form.rawOptions.refererUrl,
      })) as string;
      if (imgUrl && typeof imgUrl == 'string') {
        const filename = (await path.basename(imgUrl)) as string;
        const set = (obj: Partial<Row>) => {
          const index = tableData.value.findIndex((item) => item.url === imgUrl);
          for (const key in obj) {
            tableData.value[index][key] = obj[key];
          }
        };
        let index = tableData.value.findIndex((item) => item.url === imgUrl);
        let duplicate = 1;
        if (index == -1) {
          tableData.value.push({
            url: imgUrl,
            size: 0,
            duplicate,
            filename,
            progress: 100,
            status: '',
          });
        } else {
          duplicate = tableData.value[index].duplicate++;
        }

        if (form.rawOptions.maxDuplicate != 0 && duplicate >= form.rawOptions.maxDuplicate) {
          set({ status: 'exception' });
          stop();
          ElMessageBox.alert('已到预定出现最大次数阈值，停止运行', '提示');
          return;
        }

        if (form.rawOptions.noDownload) {
          return;
        }
        let ret: any = await invoke('download', {
          imgUrl: imgUrl,
          refUrl: form.rawOptions.refererUrl,
          filename: filename,
          output: form.outputDir,
        });
        if (ret && typeof ret == 'string') {
          try {
            ret = JSON.parse(ret);
            if (ret.ok) {
              set({
                filename: ret.filename,
                size: ret.size,
                progress: 100,
                status: 'success',
              });
            } else throw null;
          } catch {
            set({
              status: 'exception',
              progress: 0,
            });
          }
        } else {
          set({
            status: 'exception',
            progress: 0,
          });
        }
        tableRef.value?.doLayout();
      }
    });
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
};
function exportToCsv(filename, rows) {
  var processRow = function (row) {
    var finalVal = '';
    for (var j = 0; j < row.length; j++) {
      var innerValue = row[j] === null ? '' : row[j].toString();
      if (row[j] instanceof Date) {
        innerValue = row[j].toLocaleString();
      }
      var result = innerValue.replace(/"/g, '""');
      if (result.search(/("|,|\n)/g) >= 0) result = '"' + result + '"';
      if (j > 0) finalVal += ',';
      finalVal += result;
    }
    return finalVal + '\n';
  };

  var csvFile = '';
  for (var i = 0; i < rows.length; i++) {
    csvFile += processRow(rows[i]);
  }

  var blob = new Blob([csvFile], { type: 'text/csv;charset=utf-8;' });
  if ((navigator as any).msSaveBlob) {
    (navigator as any).msSaveBlob(blob, filename);
  } else {
    var link = document.createElement('a');
    if (link.download !== undefined) {
      var url = URL.createObjectURL(blob);
      link.setAttribute('href', url);
      link.setAttribute('download', filename);
      link.style.visibility = 'hidden';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
  }
}
const tapExportCsv = async () => {
  let head = [['图片链接', '文件名', '图片大小']];
  if (form.rawOptions.noDownload) head = [head[0].slice(0, 2)];
  const data = head.concat(
    tableData.value.map((item) => {
      let t = [item.url, item.filename];
      if (!form.rawOptions.noDownload) t.push(prettyBytes(item.size));
      return t;
    }) as any[]
  );
  exportToCsv('export.csv', data);
};
const tapReset = async () => {
  tableData.value = [];
  tableRef.value?.doLayout();
  queue = null;
};
const tapButton = async (formEl: FormInstance | undefined) => {
  if (running.value) {
    stop();
  } else {
    formEl &&
      formEl.validate((valid) => {
        if (valid) {
          start();
        }
      });
  }
};
const tapOutputDir = async () => {
  const ret = await dialog.open({
    defaultPath: form.outputDir,
    directory: true,
  });
  if (ret && typeof ret == 'string') {
    form.outputDir = ret;
  }
};
const filterStatus = (value: string, row: Row) => {
  if (value == 'failed') {
    return row.progress == 0;
  }
  if (value == 'success') {
    return row.progress == 100;
  }
};
const openInExplorer = async (filename) => {
  const p = await path.join(form.outputDir, filename);
  const c = new shell.Command('explorer', ['/select,' + p]);
  c.execute();
};
</script>
