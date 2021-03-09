# 下载随机图片接口的图片


## 起因

在某个夜晚，我正在聊QQ，某个人在群里发了一个网站，背景是一张找了很久的图片，然而，劳资用的是手机，而且在QQ中打开，并保存不了，重新打开图已不见，流下了没技术的眼泪。然而夜生活才刚刚开始。

在死循环的边缘试探。死循环请求接口，取得location，获取文件名，判断是否有下载，如果循环中频繁取得同一张图片地址大于设置值，可以判断是否已将人家接口榨干。
## 说明

仅适用于直接跳转的随机图片接口。比如一个数组弄一堆上传到新浪图床的地址，然后随机取出跳转。

他们的接口十分神秘，但你又想得到所有图片。我非常不建议做这种消耗人家资源的不人道行为 滑稽

出现任何问题请不要联系我 

环境：php>7.0 php-cli

## 使用
请在cmd或shell 执行此命令

```shell
git clone https://github.com/qiangmouren/download-api-images
# 如果未安装git 请执行 yum install git -y
cd download_all_images_of_api
php download.php
```

![aa](https://imgtu.com/i/63NK5q)
