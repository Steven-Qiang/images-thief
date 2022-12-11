<template>
  <div style="user-select: none" ref="header">
    <h2 style="margin-left: 50px">Images Thief 图片小偷</h2>
    <h5 style="margin-left: 63px">
      <span style="vertical-align: middle">批量下载随机图片接口的所有图片</span>
      <img
        src="https://visitor-badge.glitch.me/badge?page_id=images-thief"
        loading="lazy"
        style="vertical-align: middle; margin-left: 10px"
      />
    </h5>
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
        <el-checkbox-group v-model="form.options" :disabled="running">
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
              label="setOnlyRecord"
              @change="
                () => {
                  form.rawOptions.onlyRecord = form.options.includes('setOnlyRecord');
                }
              "
              >不下载图片</el-checkbox
            >
          </el-tooltip>
        </el-checkbox-group>
      </el-form-item>
      <div style="display: flex">
        <el-form-item label="并发数" v-if="isSetConcurrency" prop="rawOptions.concurrency">
          <el-input-number v-model="form.rawOptions.concurrency" style="width: 150px" :min="1" />
        </el-form-item>
        <el-form-item label="设置阈值" v-if="isSetMaxDuplicate" prop="rawOptions.maxDuplicate">
          <el-input-number v-model="form.rawOptions.maxDuplicate" style="width: 150px" :min="0" />
        </el-form-item>
      </div>
      <el-form-item label="来源地址" v-if="isSetRefererUrl" prop="rawOptions.refererUrl">
        <el-input v-model="form.rawOptions.refererUrl" style="max-width: 500px" />
      </el-form-item>
      <el-form-item>
        <el-button :type="running ? 'danger' : 'primary'" @click="tapButton(formRef)">{{ btnText }}</el-button>
        <el-button type="warning" v-if="tableData.length" :disabled="running" @click="tapReset">重置列表</el-button>
        <el-button type="info" v-if="tableData.length" :disabled="running" @click="tapExportCsv">导出列表</el-button>
        <div style="margin-left: 20px">
          发现数量：{{ tableData.length }} ，完成数量：{{
            tableData.filter((x) => x.progress == 100).length
          }}
          ，总耗时：{{ time }}秒
        </div>
      </el-form-item>
    </el-form>
  </div>
  <div class="table" :style="{ marginLeft: '50px', marginRight: '20px', width: 'calc(100% - 70px)' }">
    <el-table
      ref="tableRef"
      row-key="url"
      :data="tableData.slice((currentPage - 1) * pageSize, currentPage * pageSize)"
      :stripe="true"
      :border="true"
      :style="{ height: tableHeight }"
      @sort-change="tapSortChange"
      :default-sort="{ prop: 'startTime', order: 'descending' }"
    >
      <el-table-column prop="url" label="图片地址" sortable />
      <el-table-column prop="filename" label="文件名" sortable>
        <template #default="scope">
          <a href="#" @click="openInExplorer(scope.row.filename)" v-if="!form.rawOptions.onlyRecord">{{
            scope.row.filename
          }}</a>
          <span v-else>{{ scope.row.filename }}</span>
        </template>
      </el-table-column>
      <el-table-column
        prop="size"
        label="文件大小"
        width="180"
        v-if="!form.rawOptions.onlyRecord"
        sortable
        align="center"
      >
        <template #default="scope">
          <span>{{ prettyBytes(scope.row.size) }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="duplicate" label="重复次数" width="120" sortable align="center">
        <template #default="scope">
          <span>{{ scope.row.duplicate }}次</span>
        </template>
      </el-table-column>
      <el-table-column
        fixed="right"
        prop="progress"
        label="进度"
        width="180"
        v-if="!form.rawOptions.onlyRecord"
        align="center"
      >
        <template #default="scope">
          <el-progress
            :percentage="scope.row.progress"
            :format="progressBarFormat"
            :color="progressBarCustomColorMethod"
          />
        </template>
      </el-table-column>
      <el-table-column prop="startTime" label="开始时间" width="180" sortable align="center">
        <template #default="scope">
          <span>{{ scope.row.startTime }}</span>
        </template>
      </el-table-column>
    </el-table>
    <div style="margin: 10px 0">
      <el-pagination
        :page-size="pageSize"
        layout="prev, pager, next"
        :total="tableData.length"
        @current-change="tapPagination"
      >
      </el-pagination>
    </div>
  </div>
  <div class="footer" ref="footer">
    @{{ new Date().getFullYear() }}&nbsp;
    <a @click="shell.open('https://github.com/qiangmouren/images-thief')" target="_blank" rel="noopener noreferrer"
      >Qiangmouren/images-thief</a
    >
  </div>
</template>

<script lang="ts" setup>
import { reactive, ref, watch, onMounted, computed } from 'vue';
import { ElMessageBox, FormInstance, FormRules } from 'element-plus';
import { ElTable } from 'element-plus';
import * as fs from '@tauri-apps/api/fs';
import * as path from '@tauri-apps/api/path';
import * as dialog from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import * as shell from '@tauri-apps/api/shell';
import { listen } from '@tauri-apps/api/event';
import { nanoid } from 'nanoid';

import exportToCsv from './util/exportToCsv';
import prettyBytes from 'pretty-bytes';
import Queue from 'p-queue';

interface Row {
  uniqueId: string;
  url: string;
  size: number;
  duplicate: number;
  filename: string;
  progress: number;
  startTime: string;
}

const isdev = import.meta.env.DEV;
const tableRef = ref<InstanceType<typeof ElTable>>();
const tableData = ref<Row[]>([]);
let currentPage = ref(1);
let pageSize = ref(20);

let time = ref(0);
let btnText = ref('开始');
let running = ref<boolean>(false);
const formSize = ref('default');
const formRef = ref<FormInstance>();
const form = reactive({
  apiUrl: isdev ? 'http://www.lxh5068.com/tapi/acgurl.php' : '',
  outputDir: '',
  options: ['setConcurrency', 'setRefererUrl'] as string[],
  rawOptions: {
    refererUrl: '',
    maxDuplicate: 3,
    onlyRecord: false,
    concurrency: 5,
  },
});

const isSetRefererUrl = computed(() => form.options.includes('setRefererUrl'));
const isSetMaxDuplicate = computed(() => form.options.includes('setMaxDuplicate'));
const isSetOnlyRecord = computed(() => form.options.includes('setOnlyRecord'));
const isSetConcurrency = computed(() => form.options.includes('setConcurrency'));

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

const header = ref<Element>();
const footer = ref<Element>();
const tableHeight = computed(() => {
  const $header = header.value?.getBoundingClientRect();
  const $footer = footer.value?.getBoundingClientRect();

  const headerHeight = $header?.height || 0;
  const footerHeight = $footer?.height || 0;
  const minHeight = window.innerHeight - headerHeight - footerHeight - 110;
  return minHeight + 'px';
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

listen('progress', (event) => {
  if (typeof event.payload == 'string') {
    let { unique_id, progress } = JSON.parse(event.payload);
    progress = parseInt(progress);
    const index = tableData.value.findIndex((item) => item.uniqueId === unique_id);
    if (index > -1) {
      tableData.value[index].progress = progress;
      tableRef.value?.doLayout();
    }
  }
});
const progressBarFormat = (percentage) => {
  return percentage === 100 ? '完成' : `${percentage}%`;
};
const progressBarCustomColorMethod = (percentage) => {
  if (percentage < 30) {
    return '#909399';
  } else if (percentage < 70) {
    return '#e6a23c';
  } else {
    return '#67c23a';
  }
};
const createQueue = () => {
  const concurrency = parseInt(form.rawOptions.concurrency.toString());
  return new Queue({
    concurrency,
    interval: 100,
  });
};
/**
 * 获取接口重定向数据
 * 返回包括 图片网址 大小 文件名
 */
const getRedirectInfo = async (): Promise<{ url: string; size: number; filename: string } | null> => {
  return invoke('get', {
    url: form.apiUrl,
    referer: form.rawOptions.refererUrl,
  }).then(($resp) => {
    if (!$resp) return null;
    const json = JSON.parse($resp + '');
    return json;
  });
};
const checkUrlRedirection = async () => {
  const resp = await getRedirectInfo();
  if (!resp) {
    ElMessageBox.alert('该链接似乎不存在跳转，请检查URL是否正确', '提示');
    return false;
  }
  return true;
};

let timer: number | null | NodeJS.Timer;
let queue: Queue | null;
const stopQueue = async () => {
  btnText.value = '停止中...';
  if (timer) {
    // 停止计时器
    clearTimeout(timer);
    timer = null;
  }
  if (queue) {
    // 停止线程
    queue.pause();
    queue.clear();
    queue = null;
  }
  running.value = false;
  btnText.value = '开始';
};
const startQueue = async () => {
  // 创建计时器
  time.value = 0;
  timer = setInterval(() => {
    time.value++;
  }, 1000);
  running.value = true;
  btnText.value = '停止';

  // 检测链接是否存在跳转
  if (!(await checkUrlRedirection())) {
    return stopQueue();
  }
  // 创建目录
  await fs.createDir(form.outputDir, { recursive: true }).catch(console.log);

  queue = createQueue();
  while (running.value) {
    queue.add(async () => {
      const redirectInfo = await getRedirectInfo();
      const uniqueId = nanoid();
      if (redirectInfo && redirectInfo.url) {
        const { url, size, filename } = redirectInfo;
        let index = tableData.value.findIndex((item) => item.uniqueId === uniqueId);
        let duplicate = 1;
        if (index == -1) {
          // 初始值
          tableData.value.push({
            startTime: new Date().toLocaleString('lt'),
            uniqueId,
            url,
            size: Number(size),
            filename,
            duplicate,
            progress: 0,
          });
        } else {
          // 重复值
          duplicate = tableData.value[index].duplicate++;
        }

        if (form.rawOptions.maxDuplicate != 0 && duplicate >= form.rawOptions.maxDuplicate) {
          // 超过阈值
          // tableData.value[index].status = 'exception';
          await stopQueue();
          ElMessageBox.alert('已超过预定阈值,停止运行', '提示');
          return;
        }
        if (!form.rawOptions.onlyRecord) {
          // 下载
          invoke('download', {
            url,
            referer: form.rawOptions.refererUrl,
            filename,
            size,
            path: form.outputDir,
            uniqueId,
          });
        }
        tableRef.value?.doLayout();
      }
    });
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
};

const tapExportCsv = async () => {
  let head = [['图片链接', '文件名', '图片大小']];
  if (form.rawOptions.onlyRecord) head = [head[0].slice(0, 2)];
  const data = head.concat(
    tableData.value.map((item) => {
      let t = [item.url, item.filename];
      if (!form.rawOptions.onlyRecord) t.push(prettyBytes(item.size));
      return t;
    }) as any[]
  );
  exportToCsv('export.csv', data);
};

const tapReset = async () => {
  tableData.value = [];
  tableRef.value?.doLayout();
  time.value = 0;
  queue = null;
};

const tapButton = async (formEl: FormInstance | undefined) => {
  if (running.value) {
    await stopQueue();
  } else {
    formEl &&
      formEl.validate((valid) => {
        if (valid) {
          startQueue();
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

const tapPagination = (_currentPage: number) => {
  currentPage.value = _currentPage;
  tableRef.value?.doLayout();
};

const tapSortChange = ({ prop, order }: { prop: string; order: string }) => {
  if (order == 'ascending') {
    tableData.value = tableData.value.sort((a, b) => {
      if (a[prop] > b[prop]) return 1;
      if (a[prop] < b[prop]) return -1;
      return 0;
    });
  } else if (order == 'descending') {
    tableData.value = tableData.value.sort((a, b) => {
      if (a[prop] > b[prop]) return -1;
      if (a[prop] < b[prop]) return 1;
      return 0;
    });
  }
  tableRef.value?.doLayout();
};

const openInExplorer = async (filename) => {
  const p = await path.join(form.outputDir, filename);
  const c = new shell.Command('explorer', ['/select,' + p]);
  c.execute();
};
</script>
