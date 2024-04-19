-- fansland_sol.chat_history definition

CREATE TABLE `chat_history` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `idol_id` int NOT NULL COMMENT '偶像ID',
  `msg_id` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'msg_id, 唯一',
  `ref_msg_id` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT '回复的msg_id',
  `role` varchar(10) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '角色类型: user:用户, assistant:偶像',
  `address` varchar(60) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '用户的钱包地址',
  `content` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '聊天内容',
  `create_time` datetime DEFAULT CURRENT_TIMESTAMP,
  `update_time` datetime DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uidx_msg_id` (`msg_id`) USING BTREE,
  KEY `idx_idol_id_address` (`idol_id`,`address`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci  COMMENT='聊天记录';


-- fansland_sol.ai_idol_point_record definition

CREATE TABLE `ai_idol_point_record` (
  `id` varchar(40) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'PK',
  `idol_id` bigint NOT NULL COMMENT '偶像ID',
  `amount` bigint DEFAULT '0' COMMENT '消费数量',
  `address` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT '地址',
  `base_fee` bigint DEFAULT '0' COMMENT '平台手续费',
  `idol_pool_fee` bigint DEFAULT '0' COMMENT '偶像池手续费',
  `trans_type` int DEFAULT NULL COMMENT '交易类型, 11:文字聊天消耗',
  `create_time_stamp` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间戳(ms)',
  `create_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `create_date` date   DEFAULT (CURRENT_DATE) COMMENT '日期',
  PRIMARY KEY (`id`),
  KEY `address_fk` (`address`) USING BTREE,
  KEY `idol_fk` (`idol_id`) USING BTREE,
  KEY `idol_address_fk` (`idol_id`,`address`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='偶像积分消费记录';

-- fansland_sol.user_integral_wallet definition

CREATE TABLE `user_integral_wallet` (
  `id` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'PK',
  `address` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '地址',
  `balance` bigint DEFAULT '0' COMMENT '余额',
  `lock_balance` bigint DEFAULT '0' COMMENT '锁定余额',
  `create_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '更新时间',
  `version` int DEFAULT NULL COMMENT '版本号',
  `remark` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT '备注',
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `address_unq` (`address`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='用户积分钱包';