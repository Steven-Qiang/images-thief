<?php

/*
* 死循环下载接口图片
* @author QangMouRen<2962051004@qq.com>
* @github https://github.com/QiangMouRen/foreach_download_api_images
* @version 1.0
*/

set_time_limit(0);

if (!is_file('cache.json'))
    file_put_contents('cache.json', '[]');
$cacheData = json_decode(file_get_contents('cache.json'), 1);

$repeatMax = 3; // 单张图片重复最大次数 超过即停止循环
$imagePath = './images/'; // 图片存档目录
$api = 'http://api.5xbl.cn/api/api.php'; // 测试接口地址
$refer = "https://www.baidu.cpm/"; // 来源

if (!is_dir($imagePath))
    mkdir($imagePath);
echo "开始对[{$api}]的图片进行下载" . PHP_EOL;
$succCount = 0;
for (;;) { // 在对美图的渴望中循环
    
    $ch = curl_init();
    curl_setopt($ch, CURLOPT_URL, $api);
    curl_setopt($ch, CURLOPT_HEADER, 0);
    curl_setopt($ch, CURLOPT_NOBODY, true);
    curl_setopt($ch, CURLOPT_CUSTOMREQUEST, 'GET');
    curl_setopt($ch, CURLOPT_TIMEOUT, 20);
    curl_setopt($ch, CURLOPT_REFERER, $refer);
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
    curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, false);
    $ret = curl_exec($ch);
    $info = curl_getinfo($ch);
    $imgurl = $info['redirect_url'];
    curl_close($ch);
    if (is_array($imgurl)) { // 部分情况出现为数组
        $imgurl = end($imgurl);
    }
    
    if (!isset($imgurl)) { // 如果找不到location
        exit("停止： 接口非301/302跳转");
    }
    
    // 发现部分所谓的接口图片全部存本地
    // 跳转的是路劲，导致无法下载
    
    if (!preg_match('/(http:\/\/)|(https:\/\/)/i', $imgurl)) { // 判断是否以上情况
        if (!preg_match('/\/$/i', $api)) {
            $imgurl = $api . '/' . $imgurl; // 拼接网址
        }
        
    }
    
    
    $fileName = pathinfo($imgurl, PATHINFO_BASENAME);
    
    if (!is_file($imagePath . $fileName)) {
        $succCount++;
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $imgurl);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
        curl_setopt($ch, CURLOPT_CONNECTTIMEOUT, 30);
        $imgInfo = curl_getinfo($ch);
        $file = curl_exec($ch);
        curl_close($ch);
        $resource = fopen($imagePath . $fileName, 'a');
        fwrite($resource, $file);
        fclose($resource);
        echo "下载成功：{$imgurl} " . round(filesize($imagePath . $fileName) / 1048576 * 100) / 100 . ' MB' . PHP_EOL;
        
        $cacheData[$fileName] = array(
            "url" => $imgurl,
            "repetitions" => 1
        );
    } else {
        if (isset($cacheData[$fileName]['repetitions']) && $cacheData[$fileName]['repetitions'] > $repeatMax) {
            exit('结束：此次执行共下载' . $succCount . '张图');
            file_put_contents('cache.json', '[]');
            break;
        } else if (isset($cacheData[$fileName])) {
            $cacheData[$fileName]['repetitions']++;
        }
    }
    file_put_contents('cache.json', json_encode($cacheData, 1));
}
