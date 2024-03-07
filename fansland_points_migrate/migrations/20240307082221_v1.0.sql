-- Add migration script here
-- fansland.tb_test definition

CREATE TABLE `integral_request_record` (
  `id` varchar(32) NOT NULL COMMENT 'PK',
  `app_id` varchar(32) DEFAULT NULL COMMENT '渠道ID',
  `biz_id` varchar(32) DEFAULT NULL COMMENT '业务ID',
  `events_id` int(10) DEFAULT '0' COMMENT '活动ID',
  `goods_id` int(10) DEFAULT '0' COMMENT '商品ID',
  `request_type` int(2) NOT NULL DEFAULT '0' COMMENT '类型0购买NFT1邀请赠送2空投赠送',
  `hash` varchar(100) NOT NULL COMMENT '交易hash',
  `chain_id` varchar(50) NOT NULL COMMENT '链ID',
  `address` varchar(100) NOT NULL COMMENT '地址',
  `status` int(1) DEFAULT '0' COMMENT '状态0初始化1处理成功2处理失败',
  `deal_msg` varchar(500) DEFAULT NULL COMMENT '处理失败原因',
  `create_time` datetime DEFAULT NULL COMMENT '创建时间',
  `update_time` datetime DEFAULT NULL COMMENT '更新时间',
  `amount` bigint(20) NOT NULL DEFAULT '0' COMMENT '请求数量',
  `contract_id` varchar(100) DEFAULT NULL COMMENT '合约/糖果机ID',
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `hash_unq` (`request_type`,`hash`) USING BTREE,
  UNIQUE KEY `biz_id_app` (`app_id`,`biz_id`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 ROW_FORMAT=DYNAMIC COMMENT='积分请求记录';



