const { app, BrowserWindow, ipcMain, dialog } = require("electron");
/**
 * Set `__static` path to static files in production
 * https://simulatedgreg.gitbooks.io/electron-vue/content/en/using-static-assets.html
 */
if (process.env.NODE_ENV !== "development") {
  global.__static = require("path").join(__dirname, "/static").replace(/\\/g, "\\\\");
}


let mainWindow;
const winURL = process.env.NODE_ENV === "development"
  ? "http://localhost:9080"
  : `file://${__dirname}/index.html`;

function createWindow () {

  /**
   * Initial window options
   */
  mainWindow = new BrowserWindow({
    useContentSize: true,
    maximizable: false,
    resizable: false,
    width: 1200,
    height: 700,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,

    }
  });

  mainWindow.loadURL(winURL);

  mainWindow.on("closed", () => {
    mainWindow = null;
  });
}

app.on("ready", createWindow);


const { Sequelize, DataTypes, Op } = require("sequelize");
const sequelize = new Sequelize("sqlite::memory:", { logging: false });
const Cache = sequelize.define("Cache", {
  uuid: { type: DataTypes.STRING, primaryKey: true, unique: true },
  url: { type: DataTypes.STRING },
  size: { type: DataTypes.BIGINT },
  filename: { type: DataTypes.STRING },
  savePath: { type: DataTypes.STRING },
  repetitions: { type: DataTypes.INTEGER },
  percent: { type: DataTypes.DECIMAL(10, 2) },
  status: { type: DataTypes.ENUM("active", "exception", "success") },
  transferred: { type: DataTypes.BIGINT },
}, { freezeTableName: true, updatedAt: true });

const util = require("util");
const stream = require("stream");
const pipeline = util.promisify(stream.pipeline);


const got = require("got");
const os = require("os");
const fs = require("fs");
const path = require("path");
const tls = require("tls");
const uuid = require("uuid").v4;
const json2csv = require("json2csv");


const delay = require("delay");
const Queue = require("p-queue").default;

tls.DEFAULT_ECDH_CURVE = "auto";
tls.DEFAULT_MIN_VERSION = "TLSv1";

/** @type {Queue} */
let _queue;
let _timer;
let _send_result;

let _stop_queue = () => {
  _timer && clearInterval(_timer);
  _queue && _queue.clear();
  _queue = null;
  _send_result && _send_result();
};

ipcMain.on("start-queue", async (
  { sender },
  { DOWNLOAD_QUEUE, DOWNLOAD_DELAY, DOWNLOAD_API, DOWNLOAD_REFERER, DOWNLOAD_PATH, DOWNLOAD_MAX, ONLY_RECORD }
) => {

  await Cache.sync({ force: true });
  await fs.promises.mkdir(DOWNLOAD_PATH, { recursive: true });

  _timer && clearInterval(_timer);
  _queue = new Queue({ concurrency: parseInt(DOWNLOAD_QUEUE) });
  _send_result = async () => {
    sender.send("queue-result", {
      processing: !!_queue,
      list: await Cache.findAll({ raw: true, order: [["updatedAt", "DESC"]] })
    });
  };
  _timer = setInterval(_send_result, 100);

  const headers = {
    "referer": DOWNLOAD_REFERER || DOWNLOAD_API,
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.131 Safari/537.36",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9",
    "Accept-Language": "zh-CN,zh;q=0.9,en;q=0.8",
    "sec-ch-ua": "\"Chromium\";v=\"92\", \" Not A;Brand\";v=\"99\", \"Google Chrome\";v=\"92\"",
    "sec-ch-ua-mobile": "?0",
  };

  const _fetch = async () => {
    const resp = await got(DOWNLOAD_API, { method: "GET", followRedirect: false, headers });
    if (!resp.headers.location) {
      _stop_queue();
      sender.send("notify", {
        type: "error",
        message: `接口无跳转。状态码：${resp.statusCode}`
      });
      return;
    }
    return resp;
  };

  try {
    await _fetch();
  } catch (error) {
    _stop_queue();
    return sender.send("notify", {
      type: "error",
      message: `请求接口出错。${error}`
    });

  }

  while (1) {
    if (!_queue) break;
    _queue.add(async () => {

      await delay(parseInt(DOWNLOAD_DELAY * 1000));

      const _id = uuid();
      if (DOWNLOAD_MAX > 0 && await Cache.count({ where: { repetitions: { [Op.gt]: DOWNLOAD_MAX } } }) > 0) {
        _stop_queue();
        return sender.send("notify", {
          type: "error",
          message: "超过最大出现次数。已停止。"
        });
      }

      const resp = await _fetch();
      if (!resp) return;

      const redirectBuffer = Buffer.from(resp.headers.location, "binary").toString();
      const whatwg = new URL(redirectBuffer, DOWNLOAD_API);
      const parse = path.parse(whatwg.pathname);
      const imgURL = whatwg.toString();
      const savePath = path.join(DOWNLOAD_PATH, parse.base);

      if (await Cache.count({ where: { filename: parse.base } }) > 0) {
        await Cache.increment("repetitions", { where: { filename: parse.base } });
        return;
      }
      if (fs.existsSync(savePath)) return;

      await Cache.create({
        uuid: _id,
        url: imgURL,
        filename: parse.base,
        savePath,
        size: 0,
        repetitions: 1,
        percent: 0,
        status: "active",
        transferred: 0
      });
      if (ONLY_RECORD) {
        await Cache.update(
          {
            percent: 1,
            size: 0,
            status: "success",
            transferred: 0
          },
          { where: { uuid: _id } }
        );
        return;
      }
      try {
        const _resp = got.stream(imgURL, { headers });
        _resp.on("downloadProgress", async ({ percent, total, transferred }) => {
          await Cache.update(
            {
              percent,
              size: total,
              status: percent < 1 ? "active" : "success",
              transferred
            },
            { where: { uuid: _id } }
          );
        });
        await pipeline(_resp, fs.createWriteStream(savePath));
      } catch (error) {
        console.log(error);
        await Cache.update(
          { status: "exception", },
          { where: { uuid: _id } }
        );
      }
    });
    await delay(100);
  }
});


ipcMain.on("stop-queue", () => {
  if (_queue) {
    _stop_queue && _stop_queue();
  }
});

ipcMain.on("export-record", async () => {
  const p = await dialog.showSaveDialog({
    filters: [{ name: "export", extensions: ["csv"] }],
    defaultPath: "export.csv"
  });

  if (p.canceled) return;
  const data = await Cache.findAll({ raw: true }).then(y =>
    y.map(x => ({
      "网址": x.url,
      "文件名": x.filename,
      "保存路径": x.savePath,
      "重复次数": x.repetitions,
      "大小": x.size,
      "状态": x.status,
      "百分比": parseInt(x.percent * 100)
    }))
  );
  const fields = ["网址", "文件名", "保存路径", "重复次数", "大小", "状态", "百分比"];
  const csv = json2csv({
    data,
    fields
  });

  await fs.promises.writeFile(p.filePath, csv, { encoding: "utf8" });
});


ipcMain.on("get-config", ({ sender }) => {
  sender.send("got-config", {
    desktopDir: path.join(os.homedir(), "Desktop")
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  if (mainWindow === null) {
    createWindow();
  }
});