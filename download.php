<?php
/*
 * @file: download.php
 * @description: 循环下载随机图片接口的图片
 * @package: download-api-images-master
 * @create: 2021-02-28 05:02:54
 * @author: qiangmouren (2962051004@qq.com)
 * @github: https://github.com/qiangmouren/download-api-images
 * -----
 * @last-modified: 2021-03-09 03:11:32
 * -----
 */

define('LAZER_DATA_PATH', realpath(__DIR__) . '/cache/');
define('CACHE_TABLE', 'images_cache'); // 缓存目录 请勿修改


define('DOWNLOAD_API', 'https://api.ixiaowai.cn/api/api.php'); // 随机图片接口地址
define('DOWNLOAD_REFER', ''); // 来源地址，避过接口来源检测 默认为接口地址


define('DOWNLOAD_RETRY', 3); // 单张图片重复最大次数 超过即停止循环
define('DOWNLOAD_PATH', 'images'); // 图片存档目录
define('DOWNLOAD_DELAY', 0); // 每次下载延迟几秒


require_once './vendor/autoload.php';

use Lazer\Classes\Database as Database;
use Lazer\Classes\Helpers\Validate;
use Lazer\Classes\LazerException;

if (!is_dir(LAZER_DATA_PATH)) mkdir(LAZER_DATA_PATH);
if (!is_dir(DOWNLOAD_PATH)) mkdir(DOWNLOAD_PATH);


try {
    Validate::table(CACHE_TABLE)->exists();
    Database::remove(CACHE_TABLE);
} catch (LazerException $e) {
}

Database::create(CACHE_TABLE, array(
    'url' => 'string',
    'file' => 'string',
    'repetitions' => 'integer'
));


set_time_limit(0);
echo "开始对[" . DOWNLOAD_API . "]的图片进行下载" . PHP_EOL;

$success_count = 0;
for (;;) { // 在对美图的渴望中循环
    sleep(DOWNLOAD_DELAY);

    $ch = curl_init();
    curl_setopt($ch, CURLOPT_URL, DOWNLOAD_API);
    curl_setopt($ch, CURLOPT_HEADER, 0);
    curl_setopt($ch, CURLOPT_NOBODY, true);
    curl_setopt($ch, CURLOPT_CUSTOMREQUEST, 'GET');
    curl_setopt($ch, CURLOPT_TIMEOUT, 20);
    curl_setopt($ch, CURLOPT_REFERER, DOWNLOAD_REFER ?? DOWNLOAD_API);
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
    curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, false);
    $ret = curl_exec($ch);
    $info = curl_getinfo($ch);
    curl_close($ch);

    $redirect_url = $info['redirect_url'];
    if (is_array($redirect_url)) { // 部分情况出现为数组
        $redirect_url = end($redirect_url);
    }

    if (!$redirect_url) { // 如果找不到location
        exit("停止： 接口非301/302跳转");
    }

    // 发现部分所谓的接口图片全部存本地
    // 跳转的是路径，导致无法下载

    if (!preg_match('/(http:\/\/)|(https:\/\/)/i', $redirect_url)) { // 判断是否以上情况
        if (!preg_match('/\/$/i', DOWNLOAD_API)) {
            $redirect_url = DOWNLOAD_API . '/' . $redirect_url; // 拼接网址
        }
    }


    $fileName = pathinfo($redirect_url, PATHINFO_BASENAME);
    $imgPath =  DOWNLOAD_PATH . '/' . $fileName;

    if (!is_file($imgPath)) {
        $success_count++;

        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $redirect_url);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
        curl_setopt($ch, CURLOPT_CONNECTTIMEOUT, 30);
        curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
        curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, false);
        $file = curl_exec($ch);
        curl_close($ch);

        $resource = fopen($imgPath, 'a');
        fwrite($resource, $file);
        fclose($resource);

        echo "下载成功：{$redirect_url} " . round(filesize($imgPath) / 1048576 * 100) / 100 . ' MB' . PHP_EOL;

        $row = Database::table(CACHE_TABLE);
        $row->setField('file', $fileName);
        $row->setField('url', $redirect_url);
        $row->setField('repetitions', 1);
        $row->save();
    } else {
        $row = Database::table(CACHE_TABLE)->where('file', '=', $fileName)->find(1);
        if ($row) {

            if ($row->repetitions > DOWNLOAD_RETRY) {

                Database::remove(CACHE_TABLE);
                echo '结束：此次执行共下载' . $success_count . '张图';
                exit;
            }
            $row->repetitions = $row->repetitions + 1;
            $row->save();
        }
    }
}
