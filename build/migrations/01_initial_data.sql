CREATE TABLE IF NOT EXISTS `user`
(
    `id`        int(11) NOT NULL AUTO_INCREMENT,
    `address`   varchar(256) NOT NULL DEFAULT '',
    `device_id` varchar(256) NOT NULL DEFAULT '',
    `session`   varchar(256) NOT NULL DEFAULT '',
    `secret`    varchar(256) NOT NULL DEFAULT '',
    PRIMARY KEY (`id`),
    UNIQUE KEY `adress_UNIQUE` (`address`),
    UNIQUE KEY `device_id_UNIQUE` (`device_id`),
    UNIQUE KEY `session_UNIQUE` (`session`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;