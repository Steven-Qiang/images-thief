<?php
/*
 * 死循环下载接口图片
 * Author QiangGe
 * Mail 2962051004@qq.com
 * 在某个夜晚，我正在聊QQ，某个人在群里发了一个网站，背景是一张找了很久的图片，然而，劳资用的是手机，而且在QQ中打开，并保存不了，重新打开图已不见，流下了没技术的眼泪。然而夜生活才刚刚开始。
 * 在死循环的边缘试探。死循环请求接口，取得location，获取文件名，判断是否有下载，如果循环中频繁取得同一张图片地址大于设置值，可以判断是否已将人家接口榨干。
 * 仅适用于直接跳转的随机图片接口。比如一个数组弄一堆上传到新浪图床的地址，然后随机取出跳转。
 * 他们的接口十分神秘，但你又想得到所有图片。我非常不建议做这种消耗人家资源的不人道行为 \滑稽
 * 出现任何问题请不要联系我 (一脸安详.jpg)
 */


/*
 * 请在shell命令行下执行此文件
 *
 * 执行 php download.php
 *
 */
set_time_limit(0);

if (!is_file('cache.json'))
    file_put_contents('cache.json', '[]');
$cacheData = json_decode(file_get_contents('cache.json'), 1);

$repeatMax = 3; // 单张图片重复最大次数 超过即停止循环
$imagePath = './images/'; // 图片存档目录
$api = 'http://api.5xbl.cn/api/api.php'; // 测试接口地址
$refer = "https://www.baidu.cpm/";
if (!is_dir($imagePath))
    mkdir($imagePath);

$succCount = 0;
for (;;) { // 在对美图的渴望中循环
    
    $ch = curl_init();
    curl_setopt($ch, CURLOPT_URL, $api);
    curl_setopt($ch, CURLOPT_VERBOSE, true);
    curl_setopt($ch, CURLOPT_HEADER, true);
    curl_setopt($ch, CURLOPT_NOBODY, true);
    curl_setopt($ch, CURLOPT_CUSTOMREQUEST, 'GET');
    curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($ch, CURLOPT_TIMEOUT, 20);
    curl_setopt($ch, CURLOPT_REFERER, $refer);
    curl_setopt($ch, CURLOPT_AUTOREFERER, true);
    curl_setopt($ch, CURLOPT_FOLLOWLOCATION, true);
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
    curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, false);
    $ret = curl_exec($ch);
    $info = curl_getinfo($ch);
    $imgurl = $info['url'];
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
        $file = curl_exec($ch);
        curl_close($ch);
        $resource = fopen($imagePath . $fileName, 'a');
        fwrite($resource, $file);
        fclose($resource);
        echo "下载成功：{$imgurl}" . PHP_EOL;
        
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
