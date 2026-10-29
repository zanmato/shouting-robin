-- Two Chrome crawls of test-site/shop/ with layout capture, the second after its theme change, written by scripts/screenshots/capture --record-layout.
INSERT INTO crawls (id, root_url, started_at, finished_at, config_json, render_mode, page_count, document_count, unfetched_count) VALUES (1, 'http://127.0.0.1:8000/shop/', 1792681164, 1792681171, '{"max_pages":0,"max_concurrent":5,"delay_ms":0,"timeout_seconds":30,"respect_robots_txt":true,"follow_sitemaps":true,"block_images":false,"near_duplicate_threshold":90,"content_selector":"","user_agent":null,"extra_headers":[],"include_patterns":[],"exclude_patterns":[],"crawl_subdomains":false,"list_mode":false,"seed_urls":[],"check_resources":true,"layout_capture":"loaded"}', 'chrome', NULL, NULL, NULL);
INSERT INTO crawls (id, root_url, started_at, finished_at, config_json, render_mode, page_count, document_count, unfetched_count) VALUES (2, 'http://127.0.0.1:8000/shop/', 1792681174, 1792681180, '{"max_pages":0,"max_concurrent":5,"delay_ms":0,"timeout_seconds":30,"respect_robots_txt":true,"follow_sitemaps":true,"block_images":false,"near_duplicate_threshold":90,"content_selector":"","user_agent":null,"extra_headers":[],"include_patterns":[],"exclude_patterns":[],"crawl_subdomains":false,"list_mode":false,"seed_urls":[],"check_resources":true,"layout_capture":"loaded"}', 'chrome', NULL, NULL, NULL);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (1, 1, 9, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (2, 1, 10, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (3, 1, 11, 90, 93, 96, 100);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (4, 1, 11, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (5, 1, 13, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (6, 1, 14, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (7, 1, 15, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (8, 1, 16, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (9, 1, 18, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (10, 1, 19, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (11, 1, 20, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (12, 1, 21, 89, 93, 95, 99);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (13, 2, 35, 91, 94, 97, 101);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (14, 2, 36, 92, 94, 98, 102);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (15, 2, 38, 91, 94, 97, 101);
INSERT INTO a11y_violations (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (16, 2, 40, 91, 94, 97, 101);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (1, 1, 10, NULL, 129.0, 'SEK', 'instock', 'CABLE-2M', NULL, NULL, 1, 0, 0);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (2, 1, 13, NULL, 199.0, 'SEK', 'instock', 'CHARGER-20W', NULL, NULL, 1, 0, 0);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (3, 1, 14, NULL, 990.0, 'SEK', 'instock', 'HEADPHONES-ANC', NULL, NULL, 1, 0, 0);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (4, 1, 19, NULL, 379.0, 'SEK', 'instock', 'CHARGER-65W', NULL, NULL, 1, 0, 0);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (5, 2, 30, NULL, 199.0, 'SEK', 'instock', 'CHARGER-20W', NULL, NULL, 1, 0, 0);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (6, 2, 31, NULL, 379.0, 'SEK', 'instock', 'CHARGER-65W', NULL, NULL, 1, 0, 0);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (7, 2, 34, NULL, 129.0, 'SEK', 'instock', 'CABLE-2M', NULL, NULL, 1, 0, 0);
INSERT INTO ecommerce (id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand, has_image, has_description, has_review_or_rating) VALUES (8, 2, 39, NULL, 990.0, 'SEK', 'instock', 'HEADPHONES-ANC', NULL, NULL, 1, 0, 0);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (1, 1, 1, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (2, 1, 1, 5, 87, 240, 160, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (3, 1, 1, 5, 87, 240, 160, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (4, 1, 1, 5, 87, 240, 160, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (5, 1, 1, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (6, 1, 1, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (7, 1, 1, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (8, 1, 1, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (9, 1, 1, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (10, 1, 9, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (11, 1, 10, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (12, 1, 10, 6, 58, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (13, 1, 10, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (14, 1, 10, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (15, 1, 10, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (16, 1, 10, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (17, 1, 10, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (18, 1, 10, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (19, 1, 11, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (20, 1, 11, 12, 87, 480, 240, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (21, 1, 13, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (22, 1, 13, 6, 60, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (23, 1, 13, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (24, 1, 13, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (25, 1, 13, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (26, 1, 13, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (27, 1, 13, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (28, 1, 13, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (29, 1, 13, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (30, 1, 13, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (31, 1, 14, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (32, 1, 14, 6, 61, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (33, 1, 14, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (34, 1, 14, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (35, 1, 14, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (36, 1, 14, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (37, 1, 14, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (38, 1, 14, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (39, 1, 14, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (40, 1, 15, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (41, 1, 15, 6, 62, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (42, 1, 15, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (43, 1, 15, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (44, 1, 15, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (45, 1, 15, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (46, 1, 15, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (47, 1, 15, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (48, 1, 15, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (49, 1, 16, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (50, 1, 16, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (51, 1, 16, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (52, 1, 16, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (53, 1, 16, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (54, 1, 16, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (55, 1, 16, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (56, 1, 16, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (57, 1, 16, 6, 64, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (58, 1, 16, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (59, 1, 18, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (60, 1, 19, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (61, 1, 19, 6, 64, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (62, 1, 19, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (63, 1, 19, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (64, 1, 19, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (65, 1, 19, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (66, 1, 19, 6, 87, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (67, 1, 19, 12, 87, 800, 240, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (68, 1, 19, 6, 60, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (69, 1, 19, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (70, 1, 19, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (71, 1, 19, 6, 62, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (72, 1, 20, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (73, 1, 20, 12, 46, 1200, 240, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (74, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (75, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (76, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (77, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (78, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (79, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (80, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (81, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (82, 1, 20, 6, 58, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (83, 1, 21, 4, 43, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (84, 1, 21, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (85, 1, 21, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (86, 1, 21, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (87, 1, 21, 6, 61, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (88, 2, 22, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (89, 2, 22, 25, 88, 240, 160, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (90, 2, 22, 25, 88, 240, 160, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (91, 2, 22, 25, 88, 240, 160, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (92, 2, 22, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (93, 2, 22, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (94, 2, 22, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (95, 2, 22, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (96, 2, 22, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (97, 2, 30, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (98, 2, 30, 27, 80, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (99, 2, 30, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (100, 2, 30, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (101, 2, 30, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (102, 2, 30, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (103, 2, 30, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (104, 2, 30, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (105, 2, 30, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (106, 2, 30, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (107, 2, 31, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (108, 2, 31, 27, 81, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (109, 2, 31, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (110, 2, 31, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (111, 2, 31, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (112, 2, 31, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (113, 2, 31, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (114, 2, 31, 32, 88, 800, 240, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (115, 2, 31, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (116, 2, 31, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (117, 2, 31, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (118, 2, 31, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (119, 2, 33, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (120, 2, 33, 27, 82, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (121, 2, 33, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (122, 2, 33, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (123, 2, 33, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (124, 2, 33, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (125, 2, 33, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (126, 2, 33, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (127, 2, 33, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (128, 2, 34, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (129, 2, 34, 27, 83, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (130, 2, 34, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (131, 2, 34, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (132, 2, 34, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (133, 2, 34, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (134, 2, 34, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (135, 2, 34, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (136, 2, 35, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (137, 2, 35, 32, 68, 1200, 240, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (138, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (139, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (140, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (141, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (142, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (143, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (144, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (145, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (146, 2, 35, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (147, 2, 36, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (148, 2, 36, 32, 88, 480, 240, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (149, 2, 37, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (150, 2, 38, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (151, 2, 38, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (152, 2, 38, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (153, 2, 38, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (154, 2, 38, 27, 86, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (155, 2, 39, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (156, 2, 39, 27, 86, 520, 520, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (157, 2, 39, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (158, 2, 39, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (159, 2, 39, 27, 88, 72, 72, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (160, 2, 39, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (161, 2, 39, 27, 83, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (162, 2, 39, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (163, 2, 39, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (164, 2, 40, 26, 65, 160, 40, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (165, 2, 40, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (166, 2, 40, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (167, 2, 40, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (168, 2, 40, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (169, 2, 40, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (170, 2, 40, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (171, 2, 40, 27, 80, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (172, 2, 40, 27, 81, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (173, 2, 40, 27, 82, 200, 200, 1);
INSERT INTO images (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (174, 2, 41, 26, 65, 160, 40, 1);
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 0, 'body.page');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 1, 'header.site-header');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 2, 'div.site-header__top');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 3, 'a.logo');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 4, 'img');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 5, 'form.search');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 6, 'input.search__input');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 7, 'button.search__button');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 8, 'a.cart-link');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 9, 'nav.main-nav');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 10, 'ul.main-nav__list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 11, 'li.main-nav__item');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 12, 'main.main');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 13, 'section.hero');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 14, 'h1.hero__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 15, 'p.hero__text');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 16, 'a.hero__button');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 17, 'div.category-tiles');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 18, 'a.category-tile');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 19, 'span.category-tile__name');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 20, 'section.featured');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 21, 'h2.section-title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 22, 'ul.product-grid');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 23, 'li.product-card');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 24, 'a.product-card__link');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 25, 'img.product-card__image');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 26, 'span.product-card__name');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 27, 'span.product-card__price');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 28, 'footer.site-footer');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 29, 'div.site-footer__columns');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 30, 'div.footer-column');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 31, 'h3.footer-column__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 32, 'ul.footer-column__list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 33, 'li');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 34, 'p.footer-column__text');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 35, 'p.site-footer__legal');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 36, 'nav.breadcrumbs');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 37, 'ol.breadcrumbs__list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 38, 'li.breadcrumbs__item');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 39, 'h1.info-title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 40, 'ol.steps');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 41, 'li.step');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 42, 'h2');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 43, 'p');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 44, 'details.faq');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 45, 'summary');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 46, 'div.product');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 47, 'div.gallery');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 48, 'div.gallery__main');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 49, 'ul.gallery__thumbs');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 50, 'li.gallery__thumb');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 51, 'div.buy-box');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 52, 'h1.buy-box__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 53, 'p.viewing-now');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 54, 'span.buy-box__price');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 55, 'p.stock');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 56, 'button.add-to-cart');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 57, 'ul.trust-badges');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 58, 'li.trust-badge');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 59, 'section.description');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 60, 'div.description__body');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 61, 'section.specs');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 62, 'table.specs__table');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 63, 'tbody');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 64, 'tr.specs__row');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 65, 'th.specs__name');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 66, 'td.specs__value');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 67, 'section.reviews');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 68, 'section.related');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 69, 'ul.related-list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 70, 'div.media-text');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 71, 'img.media-text__image');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 72, 'div.media-text__body');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 73, 'blockquote.quote');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 74, 'span.sale-badge');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 75, 'h1.category-title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 76, 'p.category-intro');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 77, 'div.listing');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 78, 'aside.filters');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 79, 'h2.filters__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 80, 'ul.filters__list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 81, 'li.filters__option');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 82, 'div.listing__results');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 83, 'p.listing__count');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 84, 'nav.pagination');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 85, 'div.info-columns');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 86, 'div.info-card');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 87, 'table.info-table');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 88, 'tr');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 89, 'th');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 90, 'td');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 91, 'div.manufacturer-content');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 92, 'div.manufacturer-content__banner');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 93, 'div.manufacturer-content__features');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 94, 'div.manufacturer-content__feature');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 95, 'h3');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (1, 96, 'div.category-banner');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 0, 'body.page');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 1, 'header.site-header');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 2, 'div.site-header__top');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 3, 'a.logo');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 4, 'img');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 5, 'form.search');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 6, 'input.search__input');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 7, 'button.search__button');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 8, 'a.cart-link');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 9, 'nav.main-nav');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 10, 'ul.main-nav__list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 11, 'li.main-nav__item');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 12, 'main.main');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 13, 'section.hero');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 14, 'h1.hero__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 15, 'p.hero__text');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 16, 'a.hero__button');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 17, 'div.category-tiles');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 18, 'a.category-tile');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 19, 'span.category-tile__name');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 20, 'section.featured');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 21, 'h2.section-title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 22, 'ul.product-grid');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 23, 'li.product-card');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 24, 'a.product-card__link');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 25, 'img.product-card__image');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 26, 'span.product-card__name');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 27, 'span.product-card__price');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 28, 'footer.site-footer');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 29, 'div.site-footer__columns');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 30, 'div.footer-column');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 31, 'h3.footer-column__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 32, 'ul.footer-column__list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 33, 'li');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 34, 'p.footer-column__text');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 35, 'p.site-footer__legal');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 36, 'div.product');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 37, 'div.gallery');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 38, 'div.gallery__main');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 39, 'ul.gallery__thumbs');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 40, 'li.gallery__thumb');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 41, 'div.buy-box');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 42, 'h1.buy-box__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 43, 'p.viewing-now');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 44, 'span.buy-box__price');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 45, 'p.stock');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 46, 'button.add-to-cart');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 47, 'section.description');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 48, 'div.description__body');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 49, 'p');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 50, 'section.specs');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 51, 'table.specs__table');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 52, 'tbody');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 53, 'tr.specs__row');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 54, 'th.specs__name');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 55, 'td.specs__value');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 56, 'section.reviews');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 57, 'section.related');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 58, 'ul.related-list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 59, 'span.sale-badge');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 60, 'div.manufacturer-content');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 61, 'div.manufacturer-content__banner');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 62, 'div.manufacturer-content__features');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 63, 'div.manufacturer-content__feature');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 64, 'h3');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 65, 'div.category-banner');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 66, 'h1.category-title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 67, 'p.category-intro');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 68, 'div.listing');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 69, 'aside.filters');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 70, 'h2.filters__title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 71, 'ul.filters__list');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 72, 'li.filters__option');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 73, 'div.listing__results');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 74, 'p.listing__count');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 75, 'nav.pagination');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 76, 'h1.info-title');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 77, 'div.media-text');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 78, 'img.media-text__image');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 79, 'div.media-text__body');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 80, 'blockquote.quote');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 81, 'ol.steps');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 82, 'li.step');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 83, 'h2');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 84, 'details.faq');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 85, 'summary');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 86, 'div.info-columns');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 87, 'div.info-card');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 88, 'table.info-table');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 89, 'tr');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 90, 'th');
INSERT INTO layout_keys (crawl_id, id, key) VALUES (2, 91, 'td');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 0, '["rgb(29, 29, 27)","rgb(255, 255, 255)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 1, '["rgb(255, 255, 255)","rgb(11, 42, 74)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 2, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","16px","16px","24px","24px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 3, '["rgb(11, 79, 138)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(11, 79, 138)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 4, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 5, '["rgb(0, 0, 0)","rgb(255, 255, 255)","none","Arial","13.3333px","400","normal","start","none","none","0px","rgb(0, 0, 0)","4px 0px 0px 4px","10px","10px","12px","12px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 6, '["rgb(0, 0, 0)","rgb(245, 165, 36)","none","Arial","13.3333px","400","normal","center","none","none","0px","rgb(0, 0, 0)","0px 4px 4px 0px","10px","10px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 7, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 8, '["rgb(255, 255, 255)","rgb(18, 56, 95)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 9, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","10px","10px","24px","24px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 10, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 11, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","16px","48px","24px","24px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 12, '["rgb(255, 255, 255)","rgb(11, 79, 138)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","8px","64px","64px","48px","48px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 13, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","48px","700","57.6px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 14, '["rgb(29, 29, 27)","rgb(245, 165, 36)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","6px","12px","12px","24px","24px","inline-block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 15, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","grid","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 16, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(229, 229, 224)","8px","12px","12px","12px","12px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 17, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 18, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 19, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","24px","700","36px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 20, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(229, 229, 224)","8px","12px","12px","12px","12px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 21, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","20px","700","30px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 22, '["rgb(255, 255, 255)","rgb(11, 42, 74)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","32px","32px","24px","24px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 23, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","grid","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 24, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 25, '["rgb(207, 224, 248)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","13px","400","19.5px","start","none","none","0px","rgb(207, 224, 248)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 26, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","14px","400","21px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 27, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","14px","400","21px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 28, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","14px","400","21px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 29, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","32px","700","38.4px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 30, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","16px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 31, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 32, '["rgb(29, 29, 27)","rgb(243, 243, 241)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","8px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 33, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 34, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(221, 221, 221)","4px","0px","0px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 35, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(221, 221, 221)","8px","24px","24px","24px","24px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 36, '["rgb(181, 84, 0)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","13px","400","19.5px","start","none","none","0px","rgb(181, 84, 0)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 37, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","32px","700","48px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 38, '["rgb(31, 122, 58)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(31, 122, 58)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 39, '["rgb(255, 255, 255)","rgb(31, 122, 58)","none","Arial","18px","400","normal","center","none","none","0px","rgb(255, 255, 255)","6px","14px","14px","14px","14px","inline-block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 40, '["rgb(29, 29, 27)","rgb(243, 247, 251)","none","Arial, sans-serif","14px","400","21px","start","none","none","0px","rgb(29, 29, 27)","4px","6px","6px","10px","10px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 41, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","table","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 42, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","table-row-group","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 43, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","table-row","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 44, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","left","none","none","0px","rgb(29, 29, 27)","0px","10px","10px","12px","12px","table-cell","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 45, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","left","none","none","0px","rgb(29, 29, 27)","0px","10px","10px","12px","12px","table-cell","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 46, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","24px","400","36px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 47, '["rgb(255, 255, 255)","rgb(210, 45, 45)","none","Arial, sans-serif","13px","400","19.5px","start","none","none","0px","rgb(255, 255, 255)","4px","2px","2px","8px","8px","inline-block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 48, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(229, 229, 224)","8px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 49, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","18px","700","27px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 50, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","4px","4px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 51, '["rgb(102, 102, 102)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(102, 102, 102)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 52, '["rgb(29, 29, 27)","rgb(243, 247, 251)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","8px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 53, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","left","none","none","0px","rgb(29, 29, 27)","0px","8px","8px","16px","16px","table-cell","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 54, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","left","none","none","0px","rgb(29, 29, 27)","0px","8px","8px","16px","16px","table-cell","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 55, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","4px","rgb(20, 40, 160)","0px","16px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 56, '["rgb(29, 29, 27)","rgb(238, 241, 251)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","6px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 57, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","18.72px","700","28.08px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (1, 58, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","8px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 0, '["rgb(29, 29, 27)","rgb(255, 255, 255)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 1, '["rgb(255, 255, 255)","rgb(11, 42, 74)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 2, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","16px","16px","24px","24px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 3, '["rgb(11, 79, 138)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(11, 79, 138)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 4, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 5, '["rgb(0, 0, 0)","rgb(255, 255, 255)","none","Arial","13.3333px","400","normal","start","none","none","0px","rgb(0, 0, 0)","4px 0px 0px 4px","10px","10px","12px","12px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 6, '["rgb(0, 0, 0)","rgb(245, 165, 36)","none","Arial","13.3333px","400","normal","center","none","none","0px","rgb(0, 0, 0)","0px 4px 4px 0px","10px","10px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 7, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 8, '["rgb(255, 255, 255)","rgb(18, 56, 95)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 9, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","10px","10px","24px","24px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 10, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 11, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","16px","48px","24px","24px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 12, '["rgb(255, 255, 255)","rgb(11, 79, 138)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","8px","64px","64px","48px","48px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 13, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","48px","700","60px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 14, '["rgb(29, 29, 27)","rgb(245, 165, 36)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","6px","12px","12px","24px","24px","inline-block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 15, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","grid","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 16, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(229, 229, 224)","8px","12px","12px","12px","12px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 17, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 18, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 19, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","24px","700","36px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 20, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(229, 229, 224)","8px","12px","12px","12px","12px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 21, '["rgb(208, 2, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","20px","700","30px","start","none","none","0px","rgb(208, 2, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 22, '["rgb(255, 255, 255)","rgb(255, 230, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","64px","32px","24px","24px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 23, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","grid","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 24, '["rgb(255, 255, 255)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","start","none","none","0px","rgb(255, 255, 255)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 25, '["rgb(207, 224, 248)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","13px","400","19.5px","start","none","none","0px","rgb(207, 224, 248)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 26, '["rgb(29, 29, 27)","rgb(243, 243, 241)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","8px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 27, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","flex","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 28, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(221, 221, 221)","4px","0px","0px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 29, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(221, 221, 221)","8px","24px","24px","24px","24px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 30, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","44px","700","55px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 31, '["rgb(181, 84, 0)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","13px","400","19.5px","start","none","none","0px","rgb(181, 84, 0)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 32, '["rgb(208, 2, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","32px","700","48px","start","none","none","0px","rgb(208, 2, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 33, '["rgb(31, 122, 58)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(31, 122, 58)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 34, '["rgb(255, 255, 255)","rgb(31, 122, 58)","none","Arial","18px","400","normal","center","none","none","0px","rgb(255, 255, 255)","6px","14px","14px","14px","14px","inline-block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 35, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","table","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 36, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","table-row-group","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 37, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","table-row","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 38, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","left","none","none","0px","rgb(29, 29, 27)","0px","10px","10px","12px","12px","table-cell","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 39, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","left","none","none","0px","rgb(29, 29, 27)","0px","10px","10px","12px","12px","table-cell","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 40, '["rgb(255, 255, 255)","rgb(210, 45, 45)","none","Arial, sans-serif","13px","400","19.5px","start","none","none","0px","rgb(255, 255, 255)","4px","2px","2px","8px","8px","inline-block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 41, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","4px","rgb(20, 40, 160)","0px","16px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 42, '["rgb(29, 29, 27)","rgb(238, 241, 251)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","6px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 43, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","18.72px","700","28.08px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 44, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","8px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 45, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","1px","rgb(229, 229, 224)","8px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 46, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","18px","700","27px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 47, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","4px","4px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 48, '["rgb(102, 102, 102)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(102, 102, 102)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 49, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","24px","400","36px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 50, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","16px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 51, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","0px","0px","0px","0px","0px","list-item","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 52, '["rgb(29, 29, 27)","rgb(243, 247, 251)","none","Arial, sans-serif","16px","400","24px","start","none","none","0px","rgb(29, 29, 27)","8px","16px","16px","16px","16px","block","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 53, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","700","24px","left","none","none","0px","rgb(29, 29, 27)","0px","8px","8px","16px","16px","table-cell","row","1"]');
INSERT INTO layout_styles (crawl_id, id, style_json) VALUES (2, 54, '["rgb(29, 29, 27)","rgba(0, 0, 0, 0)","none","Arial, sans-serif","16px","400","24px","left","none","none","0px","rgb(29, 29, 27)","0px","8px","8px","16px","16px","table-cell","row","1"]');
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (1, 1, 'Product', 'Product', 5);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (1, 2, 'Layout 1', NULL, 3);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (1, 3, 'Layout 2', NULL, 1);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (1, 4, 'Layout 3', NULL, 1);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (1, 5, 'Layout 4', NULL, 1);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (1, 6, 'Layout 5', NULL, 1);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (2, 1, 'Product', 'Product', 5);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (2, 2, 'Layout 1', NULL, 3);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (2, 3, 'Layout 2', NULL, 1);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (2, 4, 'Layout 3', NULL, 1);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (2, 5, 'Layout 4', NULL, 1);
INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (2, 6, 'Layout 5', NULL, 1);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (1, 1, 1, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (2, 1, 1, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (3, 1, 1, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (4, 1, 1, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (5, 1, 1, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (6, 1, 1, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (7, 1, 1, 16, 49, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (8, 1, 1, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (9, 1, 1, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (10, 1, 1, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (11, 1, 1, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (12, 1, 1, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (13, 1, 1, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (14, 1, 1, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (15, 1, 1, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (16, 1, 1, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (17, 1, 1, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (18, 1, 1, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (19, 1, 9, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (20, 1, 9, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (21, 1, 9, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (22, 1, 9, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (23, 1, 9, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (24, 1, 9, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (25, 1, 9, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (26, 1, 9, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (27, 1, 9, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (28, 1, 9, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (29, 1, 9, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (30, 1, 10, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (31, 1, 10, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (32, 1, 10, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (33, 1, 10, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (34, 1, 10, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (35, 1, 10, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (36, 1, 10, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (37, 1, 10, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (38, 1, 10, 10, 58, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (39, 1, 10, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (40, 1, 10, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (41, 1, 10, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (42, 1, 10, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (43, 1, 10, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (44, 1, 10, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (45, 1, 10, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (46, 1, 11, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (47, 1, 11, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (48, 1, 11, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (49, 1, 11, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (50, 1, 11, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (51, 1, 11, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (52, 1, 11, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (53, 1, 11, 11, 59, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (54, 1, 11, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (55, 1, 11, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (56, 1, 11, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (57, 1, 13, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (58, 1, 13, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (59, 1, 13, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (60, 1, 13, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (61, 1, 13, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (62, 1, 13, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (63, 1, 13, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (64, 1, 13, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (65, 1, 13, 13, 60, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (66, 1, 13, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (67, 1, 13, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (68, 1, 13, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (69, 1, 13, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (70, 1, 13, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (71, 1, 13, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (72, 1, 13, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (73, 1, 14, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (74, 1, 14, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (75, 1, 14, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (76, 1, 14, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (77, 1, 14, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (78, 1, 14, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (79, 1, 14, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (80, 1, 14, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (81, 1, 14, 14, 61, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (82, 1, 14, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (83, 1, 14, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (84, 1, 14, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (85, 1, 14, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (86, 1, 14, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (87, 1, 14, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (88, 1, 14, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (89, 1, 15, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (90, 1, 15, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (91, 1, 15, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (92, 1, 15, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (93, 1, 15, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (94, 1, 15, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (95, 1, 15, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (96, 1, 15, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (97, 1, 15, 15, 62, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (98, 1, 15, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (99, 1, 15, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (100, 1, 15, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (101, 1, 15, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (102, 1, 15, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (103, 1, 15, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (104, 1, 15, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (105, 1, 16, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (106, 1, 16, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (107, 1, 16, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (108, 1, 16, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (109, 1, 16, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (110, 1, 16, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (111, 1, 16, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (112, 1, 16, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (113, 1, 16, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (114, 1, 16, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (115, 1, 16, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (116, 1, 16, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (117, 1, 16, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (118, 1, 16, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (119, 1, 16, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (120, 1, 16, 19, 51, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (121, 1, 16, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (122, 1, 16, 16, 63, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (123, 1, 16, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (124, 1, 16, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (125, 1, 16, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (126, 1, 18, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (127, 1, 18, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (128, 1, 18, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (129, 1, 18, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (130, 1, 18, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (131, 1, 18, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (132, 1, 18, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (133, 1, 18, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (134, 1, 18, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (135, 1, 18, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (136, 1, 18, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (137, 1, 19, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (138, 1, 19, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (139, 1, 19, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (140, 1, 19, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (141, 1, 19, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (142, 1, 19, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (143, 1, 19, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (144, 1, 19, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (145, 1, 19, 19, 64, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (146, 1, 19, 13, 50, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (147, 1, 19, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (148, 1, 19, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (149, 1, 19, 15, 54, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (150, 1, 19, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (151, 1, 19, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (152, 1, 19, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (153, 1, 20, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (154, 1, 20, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (155, 1, 20, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (156, 1, 20, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (157, 1, 20, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (158, 1, 20, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (159, 1, 20, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (160, 1, 20, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (161, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (162, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (163, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (164, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (165, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (166, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (167, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (168, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (169, 1, 20, 10, 52, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (170, 1, 20, 20, 63, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (171, 1, 20, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (172, 1, 20, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (173, 1, 20, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (174, 1, 21, 1, 43, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (175, 1, 21, 17, 44, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (176, 1, 21, 16, 45, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (177, 1, 21, 20, 46, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (178, 1, 21, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (179, 1, 21, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (180, 1, 21, 1, 57, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (181, 1, 21, 21, 47, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (182, 1, 21, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (183, 1, 21, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (184, 1, 21, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (185, 1, 21, 14, 53, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (186, 1, 21, 21, 63, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (187, 1, 21, 18, 48, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (188, 1, 21, 9, 55, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (189, 1, 21, 11, 56, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (190, 2, 22, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (191, 2, 22, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (192, 2, 22, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (193, 2, 22, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (194, 2, 22, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (195, 2, 22, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (196, 2, 22, 40, 71, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (197, 2, 22, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (198, 2, 22, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (199, 2, 22, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (200, 2, 22, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (201, 2, 22, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (202, 2, 22, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (203, 2, 22, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (204, 2, 22, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (205, 2, 22, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (206, 2, 22, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (207, 2, 22, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (208, 2, 30, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (209, 2, 30, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (210, 2, 30, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (211, 2, 30, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (212, 2, 30, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (213, 2, 30, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (214, 2, 30, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (215, 2, 30, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (216, 2, 30, 30, 80, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (217, 2, 30, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (218, 2, 30, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (219, 2, 30, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (220, 2, 30, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (221, 2, 30, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (222, 2, 30, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (223, 2, 30, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (224, 2, 31, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (225, 2, 31, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (226, 2, 31, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (227, 2, 31, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (228, 2, 31, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (229, 2, 31, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (230, 2, 31, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (231, 2, 31, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (232, 2, 31, 31, 81, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (233, 2, 31, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (234, 2, 31, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (235, 2, 31, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (236, 2, 31, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (237, 2, 31, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (238, 2, 31, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (239, 2, 31, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (240, 2, 33, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (241, 2, 33, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (242, 2, 33, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (243, 2, 33, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (244, 2, 33, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (245, 2, 33, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (246, 2, 33, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (247, 2, 33, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (248, 2, 33, 33, 82, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (249, 2, 33, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (250, 2, 33, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (251, 2, 33, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (252, 2, 33, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (253, 2, 33, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (254, 2, 33, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (255, 2, 33, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (256, 2, 34, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (257, 2, 34, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (258, 2, 34, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (259, 2, 34, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (260, 2, 34, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (261, 2, 34, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (262, 2, 34, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (263, 2, 34, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (264, 2, 34, 34, 83, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (265, 2, 34, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (266, 2, 34, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (267, 2, 34, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (268, 2, 34, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (269, 2, 34, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (270, 2, 34, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (271, 2, 34, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (272, 2, 35, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (273, 2, 35, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (274, 2, 35, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (275, 2, 35, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (276, 2, 35, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (277, 2, 35, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (278, 2, 35, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (279, 2, 35, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (280, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (281, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (282, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (283, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (284, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (285, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (286, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (287, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (288, 2, 35, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (289, 2, 35, 35, 84, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (290, 2, 35, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (291, 2, 35, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (292, 2, 35, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (293, 2, 36, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (294, 2, 36, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (295, 2, 36, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (296, 2, 36, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (297, 2, 36, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (298, 2, 36, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (299, 2, 36, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (300, 2, 36, 36, 85, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (301, 2, 36, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (302, 2, 36, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (303, 2, 36, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (304, 2, 37, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (305, 2, 37, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (306, 2, 37, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (307, 2, 37, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (308, 2, 37, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (309, 2, 37, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (310, 2, 37, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (311, 2, 37, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (312, 2, 37, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (313, 2, 37, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (314, 2, 37, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (315, 2, 38, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (316, 2, 38, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (317, 2, 38, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (318, 2, 38, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (319, 2, 38, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (320, 2, 38, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (321, 2, 38, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (322, 2, 38, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (323, 2, 38, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (324, 2, 38, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (325, 2, 38, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (326, 2, 38, 39, 75, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (327, 2, 38, 38, 84, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (328, 2, 38, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (329, 2, 38, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (330, 2, 38, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (331, 2, 39, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (332, 2, 39, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (333, 2, 39, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (334, 2, 39, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (335, 2, 39, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (336, 2, 39, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (337, 2, 39, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (338, 2, 39, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (339, 2, 39, 39, 86, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (340, 2, 39, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (341, 2, 39, 34, 74, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (342, 2, 39, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (343, 2, 39, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (344, 2, 39, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (345, 2, 39, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (346, 2, 39, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (347, 2, 40, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (348, 2, 40, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (349, 2, 40, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (350, 2, 40, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (351, 2, 40, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (352, 2, 40, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (353, 2, 40, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (354, 2, 40, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (355, 2, 40, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (356, 2, 40, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (357, 2, 40, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (358, 2, 40, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (359, 2, 40, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (360, 2, 40, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (361, 2, 40, 30, 72, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (362, 2, 40, 31, 73, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (363, 2, 40, 33, 76, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (364, 2, 40, 40, 84, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (365, 2, 40, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (366, 2, 40, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (367, 2, 40, 36, 78, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (368, 2, 41, 22, 65, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (369, 2, 41, 42, 66, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (370, 2, 41, 40, 67, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (371, 2, 41, 35, 68, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (372, 2, 41, 38, 69, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (373, 2, 41, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (374, 2, 41, 22, 79, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (375, 2, 41, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (376, 2, 41, 41, 70, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (377, 2, 41, 37, 77, NULL, 1, 0);
INSERT INTO links (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (378, 2, 41, 36, 78, NULL, 1, 0);
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/', X'85514F481461147FEFFB666774F6FF38BBEE6EB9BB1A88870E5F9009E6418A96EAA448E8A1C3121979290F4A041173F05076680231A98890582437DC4024A8C31E2CD64EFDA3043D0CB4946DA0040B8997E2B98DCCEA521FEFFBFD797F669879F849FDE541A57FF05CDFD0C8E0294044E4C8E43A44C6253FA10B9149C814647E64481999C04D10200811745205998C0C55643EA07197ACD4D5AB6EB7DBEDF1FAFC81A0263554420F851B23D1D8BE5AB0BF299E48363713C4132D07000032082209F088B702985E280705984849BD88FA26D357253D836B28926903D3132C5D6462C205108BC54A6C5CAEF0A4A7C2E2463D005CA42AD023C1E205C961CC7A8759F63A0CE45C0E530867B01B0092300C002227FF9596675B6430CFB2A1F56836B4869666629EF5E459CF635FCAF4A62630D525B2A1014B1B1843715E2CA0B8262CCDD20D6D51CA0746F201916369439BE725D73CB7789E2DB8446D69689BAAA19D3434539D914C57993F904C6EF2321765DE55C9ACEEC9FCAFC792AA7BB2A1196E696FD190E90A434E6FA3D8F1D5C6D2DA68FBF6FA5555757BBCCAF6FE2B116CF0EBC16030540BC28D4A4451550225A244E11F67693AF57AAEF74CC4F6F1A1431F2FC7974FA436A66F7E69DA5AB1F35FAFF2F5DF77E73A23E9DB93CE79EAB735CDD99AE6898DFE5262F73BDBB640DB383EFC8E747B2CEE237E5A68E9488C5C919D7D3F57460F2F6C5D7843BA58EA7D457CB0909F1B7F581E75F67D3F62DE5BE7C50FCEDCF55B63ADC4F40D8BCF9EBC27AD4EB55D22EECE9D7E41DC397BA7FF657471499BFAD14E9EEDF93BCC8E9D722DA83A47FBBE3D3F1B3ED66CDFD9CFF73B76DF3F', 3, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/info/returns.html', X'63BCC6D5C5CEC8E754949A98925C549A9BE493595C62C8C0C8C8C8C8CCC8C4C6C1C8C8C4CCC2CFC8C8C8C4CAC6C8C8C4C2C8C4CEC824C0C8A40412626462636462E46264E2630029636563E7E0E4E2E6E6E6E65151555353D7D0D4D286201D5D6D19593979054545102127AFA4CCC0C0C0B08C91C14081816101B31A03C3045E862F82060C13184182224F18457E3089DC611159C6F88211A48681218AC120CBE01C2303838686C60C46860E26083D81054C1B7C63676060086078C7B58CD18181814181A1808181C160031B94F98007CC58C67880698DE814BE35A22F181F084D603CC01470802960099FDB045EB7298C6E36066B44231E0845B4301AA4186C6334A8317820D4C6D920A4D520A4E5A595A4D520E4D320F48DB541E805E30D418F1B8206D85950B4467405F303A10B8C0D6C206CD0C09600260DE07C54CE03210D5028C282918B8B8B5B4A5A4646965F4E58108204E505C5C4D925D8B9B84004BB04BB2428607081938BDC8EAD0F0C9580F1E5320DAFE4CBDD70757BBFA8F3B1ECAFDBC86AD7F17D0D3AB271D52510868949275CF73BA1B9D814C4EEEDB8E45DC6F55C16C4FEB14A99E7EABB9BAB41EC52E75DB6AAEEEBCF209B55D7DDA206A24176C0CCE39AA19107A21D3678ED06D156AB2786EF913C725268C66BB0F94CF83C820EAC839FEF0000', 5, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/product/cable-2m.html', X'8D527D6C1445147F6FF6EB6E975EBB7BB7D21ED7EBB5C0795E11A76B3F68A9520B5C0821A668D4A689A64221628290DA6AA286EC1F35A948744D101B20061BBC3470081A434AF48F8B4153302655B4560B65039BB3D604343992CB2546337BB439B034CECE7BBF8F79F366B2199C907F2923BEF6DE1DCFF76CEFEDDFBD6DCBAE97FBA48EDE3D3DFDDBFB760022228744F420128E2F4506A2EB21875EE6092212AE0209923012B6CE239190C84856232D386C07E11496CA587A9A994844242823F141A18DE4F1CA8AA2284B56AC8C46A3AB1FA27C9DC11BFCC3F50D8D4D6B9A5B5A5AD6EAADB5B58FE88FAE6B7BAC7D7EAED737E81B9796570497DD954295E1AA4875354BE1AA9AE5000049041A0138C64501AC12C8AA142C6466C0C1408E042EF38124CE20AB01E882AF10E88B8C462230E30120E4180221A30A188671088D0934268891E48C111E001E87A3488715007801002C420F97DDE6EC64993574E1732CE03BC4457A2200402FEA85DA6010D8FDC0E6C6F82261798BC464499180FDE124B6B13BC25E00A067C4DBD45EE29224A6494AFFAC3AA5CFA0AD5998261D69D271DC97B04A120731D14A537AA7AD750E20EDA16791BE416D6D38646A71538B6F8E77C6A7316E6A396F4E6631EEB13C9627275BC8BE0378601EC7C451D111B63842CC11BA1D813AC23647984647682F4C531BE54D6D93A98D73A6F6279A5A1A0B7C93A9CD90427425E5AE5FA5AE4549B7BB2327B8600923BC2564B9CBBCC5595C96A359AEF55ECED1459D943EC2D9DA389A220B6A8ADD6EA6F3FA4E616B31F664E7DEAC2CCB4A705928142A55236A758D5AA32EAF5CB1327A7F692C1653FDAAAAAAFE07E2B5AB1E9C9FAA5FF597065455D5EF4AF72D95CA255966492A972A6091716138F1CDE9AD4F95CFE9F0AEBA1FF7842737266E0EBF75BD323F555CFB89EFD6136CEDAFA9FE7AF89F83D5A626771F3C9BDF7971764D72BC59F9F827E6CF1ADF25D735FC6DDEF868F61CD3DED7A677327CD39FBFF4FABBFF34327E3D730E8B7B7D7B65F0FD679B33D7187FEEEAE0A1E3F599ABEE19F6E0074D8D199BF1816B8343FBEA3257187F4597DCDED4F1DD9AEBD1100CFB189E1AAB69AAEA7B5564DC99DDFA35C35563E9D3FB3FCCF6179F19CB837673FDDEEF8BBDDF1BADC33738E752B1B7EFED81A87BE7CAFCD4F94F4FFCC0B83C147B8961DB99CD5F306C39F9DE335F569CBFA00DFDD1C03459E07F1136175A58B8F63FE98EB1F6C9DF464FFE7CA4E95EF12F', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/info/about.html', X'63BCC6D5C0C6C8E754949A98925C549A9BE493595CA2C9C0C8C8C8C8CCC8C4C6C1C8C8C4CCC2CFC8C8C8C4CAC6C8C4C8250562323281D98C4C7C0C20695636760E4E2E6E6E6E6E1E155535357537770F6D4F19593979054545102127AFA4CCC0C0C0B08C91C14081816101B31A03C3045E862F82060C13184182224F18457E3089DC611159C6F88211A48681218AC120CBE01C2303C3077606060583772C0CFB3896313A303030283014303030186C6083321FF08019CB180F30AD117DC6BD46F405E303A1098C0798020E30052CE1739BC0EB3685D1CDC6608D68C403A18816468314836D8C0635060F84AEB13508693508697969E5693508F934083D603AC07E807102D703A6095C0B981A843CD688AE607E207481B1810D840D1AD812C0A4019C8FCA7920A4010A115890707171714B49CBC8C8F20B0A0AEA8989B34BB0737181087609764990277181938BDC8EAD0F0C9580F1E5320DAFE4CBDD70757BBFA8F3B1ECAFDBC86AD7F17D0DE29AA19107C2C8E2D5FDFFCD76983FB9822C56D7DDA206A241661CD9B8EA12880DD3E7B0C16B3788B65A3D317C8FE4919342335E9B82F84C18AEC31461B00E7EBEA36B45321B00', 4, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/product/charger-20w.html', X'9D527F685B55143EE7BE5FE9CB92F6BDE4AD6DD6A469B6C598CEF99AF567565DEDB6206348A788A5A0D4AD0E2768476D1554C6FBA34AA6439FA0B3D622B3CC50B6CE4D94D1A14890299D22744EEBB05BF6D89EB156D81C6410024EB9EFB592459DCC7BCE3DDF77BE77EEB9F73E2ECE8A1F49C4DD39F0D8A37D3B06869EDCBE75D7D38342D7407FDFD08EC17E40446490F00E44C2B0E54881B734649043015D54E778248C1F099210125AC322119088485C482A90B462DCD6E95AC23869A8A0A1978A48782428227183DD4C7094894EA7D3B96CD5EA7038BCF64E956D88B1B6AD6B6C6A6E696D8BC7E3EB95F6FAFABB94BB3774DCD359E21B954DCAE6CAAA6ADF8A9250E30FD406EBEA6808D486560200A410D420C001260CA0BB2027A9A02315BD267AF3C47B8EF5A6701E690D400F6410D427280D0661DE0140C8010442D2040831182064CA09B1586C3FC66631364B62292636C102C07D3086EAB813001E07009DA8A3158B9C9E41A4AD2DF8186D7C95D838BF889F3316AA190540BDBEDC5EEBF3013D3918CC345B94E86545C9595751029FD5A6B0839E1E7603807A8C5FA4C6328BA4304D26956C6852994743D6314DBAD2A4EBA03BA1BB126F60A25D9D54BA0DB97B18D53EF538AA2FA8863CEBD7E4A82647B74407A32F91A826E7CBF2229D330EDDA13BF2A28ED4F6E1BE7FC5697E8A37B9AD261731B95E93534D6EBBC965D0E43A6DD7E4295693EFD5E41946937F434D4EA3260F739634C6DAB32725F6FC28F4DC22E9B57AE4ED563A37C1EA5C8E39C7EA8CCEE41835C7B4DBCAD8DF949BD74C2A138C21CFA0C6D3A96A7CAF15D5BFF21B13438ED087BFF4F2455174FA56D4D4D4944B41A92E24D9B6D2BF6A75F8B6F2482422792449923CB747EBD7DC51E29247F2947B2549524AC2F24AA14A10451A842AA11A6E324E8D27BE3CBAEDC1AAA53CB0ABE1BBFEC0D9CD892BE37B2FF90B73C5B51FB8AFDD4FBF470A20C3FF1C74EDDEEB99D35736EE3EBDD09A9A6973BEFF3DD51762DFA43634FDAE5D7E6FE104CDCB9ECBECA4F8A2A770E6F9D7FE68A6FC52F60416F7FAFA7CF2CD87DBB217297FE44272FFC1C6EC05CAAF1AC9B75A9AB306E5C31793237B1AB2E729AF34936F4BEBB219CA0FFD941CED5E9BB5EEF78C22587BAAA6FBDA526F7361DB1714D74CA78FBEFC6E6EA878DF5F9AF5D1CB8C7986F2265FC04DF1C874A8A576F059BEB8EEEADC50E3F1C2CEAF8AB53DAF0C87ADBBF80B73273F3CF42DE5E248E4298A1DC7B67C42317EF8F5873EAD3E794A1EF9B589E6E41FFE23A16EDB2D0CABB824DC30D63FF0F3D4E11FDE69F9AFF927', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/product/headphones-anc.html', X'AD537D685B55143FE7BEF792F4654DFB5E92F563699AB65B8C695D5F5FBFD68FB95A5D9022D2F981A5A0CCAD1627CC8DDA223819EF8F0A71567D8A1FB52BA3961A46D77D08A374E81F41BAD12962E6AACEB5EBC2F6CC6A279BC34C4A419493A49255379878EE39E7F73BBF7BEF79F73EB8F8BDB820315B73D773CF766CEFEAD9B9ED911D2F759B5BBB7675F46CEFEE0144440E99C982C8383E0B094C090D3914D04AAA603223E35CC8901521A3253C323332115926B26C647664AB91E5216B4A4E5307C659296553EA24119909198AC86C906A6AC910AD56AB75D5DA755EAF777DB9C257A83C8DCAAAEA969ADA0D75F5F5F50DCEC6D2D28DCEFB37353DD07C3B7FC8B93927372F7FCD8AE42A70177A8A8A28B90B8B4B000042088A076088F302E899109714D0914487818E45E698E51D219C475A03D00EFB18282F10F57860DE02C0D810026361068C4D5841555503D511A61A4C3538F538AF8E0B00F0280CA2326C0580E7014067CA40768AD309446A9C80E398C4B75812E753F80597C4413E89BFA730222450F92527D92E3F1FE82A10E5A6F8B442CF482BCE65A61530E60961135D0776038072CC94A2D1550912C2301B734E978C39E7312AEB1866AD61D63A620BE89981F730D0A88C39DBA2725B2F2A1DCA382AAF2A5179D2ADC97E4DF6B7F80FA07F9AF93579316351A48858748B6E591475A4D1877DFFC029D329B3210CA021F89E500C61AB212886B0CD10E6D0109A93AEC913BC263FACC9114E937F454D0EA326BF6B4E48A3A664B487C4F6F3E6F6FF8F2C0A89FEBA7090D7853837CBEB9CCEC53925CE352695C1BB54C69C07B9A81C41CD44A168A6AD89ACFC5DDF5A44651F3D90E517228AA2357F8DCBE5CA923C5251B144A3A4606DF93AEF3D593E9F4FB24B9224D9EFF59796DD773B97EC590E49929C2BD2EA1C73AE5914299973CD7970073B3D1C387574CB93B9CBB57B47C5F42EF7B9CD81EBC3AF5F2E589A495F7BC476F331D2CBA6C247E12ECC58D87292F63876EEF96DAEA1B162DF8178CFC28650A4CEFAC97734BFA07E1DDA54FD8776EDE385135467BC32D749F89A7DE9EC9EB7FFAC217E397602D37B7E7521F8FED375B14BC49FB918FC60A42A7691F88D68F0C3DA9A589478EFA560FFDE8AD805E23946F023A93236477CF4A7E040DBFA58E26E1BAF04F75F2D8FCD121FFA26D8A796C67E245E7926F8E6C9B2D879E28A61BBB9FCDDEA7CB78DF0F054716D61F7CBA6F433DD98E9A91A5FEAFC325DFBB9461FB8C61967D335DF12C8D71FDC7D265DDBFB46AF3771CF82A599C94F47BF252EF6FB5E246C3AD6F21961FDA1779EFA3C6FF2B4DC7FB59A6AF62FFF9A91D3F80F96D8B422DD620D8F5F9938F4C3FEDA3BC55F', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/product/powerbank-10k.html', X'95527F681B75147FEF7B3F92DEAD69EF9AB36DDAA469B7C5984EBD5E7FAD5975B5BA2063C8A688B3A0CE39C726A8A3B6082AE3FEA81075E8093AEB1C32CB0C65ABDB04291DFA47902A9D2254A775D8AE3BB623D62A9B830C42409497DBE456B5CCF7EBF3799F7BEF7B5F8EC31969BC1203BDFD4F3EBEE389FEC1A7B76FDAFDDCC02E4044E490897E44C6F11548209634E4504099544144C6D52143164546133C321F32095939B25657A225C6C9542AA93C4C223211194AC802E09EE3F39749B22CCB2B56AD8EC562B7DDAEF32D064FDEDAD6DED1B9B62B994CAED3BA9B9BEFD0EE5CDF7357AF37EED13654D7D486EA9694FA70A421DAD84825D2D0B412002083A047010E713100AB1CF28A0E16921874305860C1393E98C105A419803E9847D09F221A8DC2821F80B143088C6519303621836118FBD1984163861919CE18E501E03E3888FA880C00BB00C062FA81CAAB9C5E2FD1A925F8045D7C9DB9B8E0A23EA4B9E3A110D03DC1E6A6784F6395799A33E59E0626C319ECA1BBC21E00D04F8857A9BDA24432986563DA6FD1316D016DD5C22CDB9C659B0F07525679EA2D4C75EB63DA565BDD3A84FA0E7D1CF597745B9DAB33D584A92636260612C758C2540B65058972DA6FF92D7F41B2907C1FEEFB074E8913A2236C7284B8236C7304DD11B63BC23C3A42AF1BA63AC19BEABDA63ACD99EAEF68AA5934D523AEF439E7665F46EAFBC9D77763A42094762D6194B7843C7790B7388BCB737A9EEBFE2F656E59654C1BE56C751A4D915237C56DA5AAFFDD5FDFD86A9C7EE46B7FB2244972A8AEBEBEBE42892A8D4D0AF9CAF0AAD5B19B2BE2F1B852A5288A52754BA279CDADDE50AA2A828AA2684BCA4DD5BE1A9F2451F1D5F86A61193B3592FAF2F896076BAEF591DD2DDF3F1B39B3217569E4950BE1E2AC77F658E0CAFDF4FC970EEB00FC4FA39DC063C9FE8B9C737A716D66BA4BFEF007D2178D6F32EBDBFF302F7EB07892FAB217E67712BE5C553CFDE21B7F7610BF903B89DEB3BE3E9B7EFB91AEDC79E28F9E4BEF3FDC963B47FCB29D7EA7B32367131F3A9F1EDEDB923B4BBCDA49BFABB4E6E689EB4EE0CAD2BB5D9E1D6C1B2FEEFCCAABC58BA05EBA7BCFB7C49DC52D5F10AE99CA1E7FF5FDFC20F1F6502440F8D1545367C3C0F3A27777EF6B43B1D2BDC3C5D9C98F8F7C475C1A8E3F43D87362E3A784C9A36F3EF459EDE42975F8D776EAD9BF7C3346417E63569A5B52AEB3750FFC3C71F4C7F73A97CBBF00', 1, 'layout_without_markup');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/category/chargers.html', X'C5925F48537114C7CFB9F7EECEDDB9CD7BD974CE3975D9B03DFDAC54281F46D108B39A5A244564A590412566F412B1401611C40D8286ED2124D61F57DACB88EA610F12D9535958940F1772B50872904659449CD5E29A603ED5E1FE3EE77CCFEFFBFB5DEEFDFD70423A6D41EBBADEAEBD9DFB7B8F1DDAD7DC7DB46F0F2022F2C88905881C2FD81091338846448EA4C1841C757891602614111C0417A182504D58416021B223272287127256C8ED221A0B4C92D96C3617562FF7F936356FDEB235D442D1DA565CE22C75952D15DBDCE59E8ACAAA2A82A7C2BB0C00208EC02A012EF33E00D50233320315A9699F42FB67CE3E29D8E39841F200EC0476904D208BF0005EAFF7000004208C90420873AA40968D002E17D086A0F12F059D489A7402460C7A9B5E254D7A059AA437EA55D2A45790298237C5710C004025F400001B117F955A61AE88638A4B38CEB9128E0C6A8A8A292E94E24257AC41D512BC80C1469670B46B4A7B3FB24E9644768269CA777B58F187157F93BFCF1F569AB36615C34A54D6F8477C8CAF8FF1292EC607F44FD61295B31696B5646D7705D510116282CAAB7C446011A1F16767724167A1E7DFACCA5A58C2718DD794C7181669B0B0D89123FBADE70B4DA9A14B99BF959224995D656E77B96C63B5F24A0A7995CD2ECBB263A9908B4B8C4EA324118C4E63292C126383C107C32DDB9D79EDE9AE7D76C4F36243707AF0CCEBF2B9577AEF2DEB6C2BCDD33835EAEFCDF72FA6AECF504EB4A7BF74ED4E7F1BDF95FEFAB12DFDA9A3353D4BFDC07066E0CFF7D6CC8132BDBEE709D5752E8F95F2CD87DE868ABEE3A2DEF7AE5E1DF8C04F3DFD5F6BDD570B57EBFB27CFF6FB28D3BF19BD7D639C6A295A7338F79D234DF728AF193ABFE37EE9E898127D5F479A5BEC00F2F34BC5FC58DBF6F6CED0F34B0D7F1B3F00', 2, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/info/shipping.html', X'63BCC6D5C6C6C8E754949A98925C549A9BE493595C62C9C0C8C8C8C8CCC8C4C6C1C8C8C4CCC2CFC8C8C8C4CAC6C8C8C4C2C8C4CEC824C008676A81988C4C6C8C4C8C5C8C4C7C0C20E5AC6CEC1C9C5CDCDCDCDC3C2AAA6A6AEAA1615ADA10146E1F111919111515111525232B27AFA0A80822E4E4959419181818963132182830302C6056636098C0CBF045D0806102234850E409A3C80F26913B2C22CB185F3082D4303044311864199C63646050F050D8C1D8C506A1A7F1406883652C20450C0CF39818188220D4124608FD846319A303030383024301030383C1063628F3010F98B18CF100D31AD11E9E35A22F181F084D603CC01470802960099FDB045EB7298C6E36066B44231E0845B4301AA4186C6334A8317820F487AD4148AB4148CB4B2B4FAB41C8A7416805D32BD6154CAB583D56B11A60632D63FEC608C141F398823A188330196B4457303F10BAC0D8C006C2060D6C0960D200CE47E53C10D200853D2CF0B9B8B8B8A5A4656464F94D84052148534BDBD454DBCC4CDBCC4C4C9C5D829D8B0B44B04BB04B32E0012717B91D5B1F182A01E3CB651A5EC997BBE1EAF67E51E763D95FB791D5AEE3FB1A0412431657DAFBCFF49F08BF3E88FD5D27FF49D0963FDD2076FFF3C419DFE61D9985AC3F6AD556A588FA8E0C10DBE5F5A61FC1262E8D20F6FD069FACDF663132C86AEBBA5BD44034C8AE231B575D02B1B96668E48168870D5EBB41B4D5EA89E17B248F9C149AF1DA14C467C2E7515CC03AF8F90E00', 6, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/product/charger-65w.html', X'A5537D4C5B55143FE7BE8FB6AF40F71EEDF8181F0546AD651B8F8ECFC21CA26B16B22C4C6724E8E61C489CC9DC44C88C9AE525B2A42289CF842132B220C14A18BAE91FC8A27FF40F66986619737EA0B0AED99E1DB28449D2250D89D1DC3E583A6CA6C9EE3DE7FC7EE777CE3BB9F7260F7F1686AC24A5AEEDA5175B9ADB3A0E1FDC75E8F5764343DB91968EE6F6771010111924BC119130AC0529F0310D19E4D08026B4D00AC71B90307624486225242C1203123712BC9708489291AC43928A643D92DD7A8DCE238C99867534B45311098F04052429B032DE6812CC66B3396963A1C3E1D8522CB3256E76756F2D2DAB2FAFA8ACF2783CD5B69AA2A2E79E67F7ED7FA148B76DB6C7B6D73E5E97D09EB4ED484BCFC8DCB0266465E7E4DAF3F268C8C9CD2F0000F023C8768041C601A02643449441452A5A35B44689758EB5FA711E690F40130411E45728B5DB61DE0840C820022101028484182044E180900933B8DDEE5E74CFA0BB97B87B19779871CFB300B01B06501E3203C0CB00A01250587002808F01B07BEDC3D8C5EBD89BA4A31CB6AD34D3C30A008D3A7C893A76131DC32B186074EC67758CE82807B3F4319999406F0B21668A8D4B54535C32931C97C0E9423FD6D21BC3510090CFF12B349414237E0C9031DBA473CC368F2149C50069089086E114AF9AECED416F8D3C666B0C498D9D28B7C8E328BF2D87A4F10245722992ABDED5EE9A479722454D5181FAB45135AAC6A8A022DDDDD8FD9F38C547798DDBA571CEBDB2C61DD03859E30E6A5C1035AE4E3745EA141469A7220D9814E94F54A4002AD200A7480112480A1045EA210B5C0F19E52A4739391153A4093E36609AD3BDC92F34FD66687A6812E56263556E8455B90833C0AA8CCA441839C2D4E8CADCBF9481072A63B61126244DA3C2539715FE402CCAF7F2FB9390E4A43FE1EA5F28088239734356569645B48B79F9E2EA2EC8DE585CE878C4E2743AC55451142B44D1525925EA26A63EEA2ADAB439A189A916AB288AB635617D9A21DD20083418D20D19F0807571C8FBEDD93DCFA4AFE639874A7E3C9233B3C37B67E8DD9BD9CBB3F1BD9FA7DC7D8AD63F9BCAAF8087586599392974C6A749B663C1EA9A92DCF663FC42A57FBACAFCC94FB4BEE0BEE4DF5EF697B2F8F1C2799A9BDE0CB6523C91BA7CF5ADF7FF2E4F34F3F2579E13E6AA0B4B946F6A55764E2EBE7698F2373C7B375FBA523348F9CDF0798CFFE6FB6BBE93FBAAC23728DF7FDDD73B5C1ABE4EF952C8F7614579384479E70D5FDFF192F035CAD334DF47E2D67090F2D1DF7DFD8D5BC2B1F7D976CB77EA7671788EF2C1CBBE6E7751F857CA652DE5EEDA733A9741BAF3C4D12B946B0B7B2EC4CE3B1538DB753AD211DFB734DB513ABEDCFA5DBCF647B9DABFC86857E3B5E3EF753A6277CB5E9E9DFC62F407CA853EE7AB146BCFD57F4DD173E68367BFC998BC28F5DD2EA33949F07E84DAEA4E54FDDF2BD6BB26DCB7AA9FBE3571E697531589BC6BA4995FABFD03', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/category/cables.html', X'CD925F48537114C7CFB9F7EECEDDB9CD7BD9749B73EAB2617BFA5956603E8CA01166A5B3681481A64606FDC1F22DE20622562F370892DA43482CCB91F6D008EB610F23507A080993F4E1428DA41E5AA451D6439CD5E22A513E7AB8BFCF39DFEFFDF2BBFC2E3F9C911ED9D1BEB3B7BBA3ABB3B7EFD4B1E69E73E73B001191474E2C42E478C181889C09CD4516448E0C930539F27891602594105C042FA1925043D84460518A23272287127276C8EF229A8B2C92D56AB516D76C0C06DB853DCD7BF7ED6F69A58AB69596B93DDEF2B5E280AFC25F59555D4DF0570636000024105815C06D3E08A0D9605166A02199CEB7E8FCCA39E70567021790320087819D04603302CB98D8A2081008044E00401854843482CA6902E5760378BD40BB82CEBF160C22653108504DC69851A52C4605AAB4226850298B51C1AC03329E048601A00ACE02001B177F8F7A717E48609A4BBADEFB92AE05D4150DD35C4B9A6BB9638F68B6C8758C34B2A42BA62BB17E645D2C85EC02D395F9325509A94AA829140BA9CA73FED76ACE59355495C1129D9FE2E3FCB6389FE6E27CD8F8E46C8325391BCBD974FB84A00B03425CD0788D1F10D880D0B8DE9C9C8D255D23BCAEBC4055A4C554B13D4FF647AF14BA524B77B470492549B27ACB7DBE06B94276B03A793395BCC5E19465D9B556C8A56566B759920866B7D903FFA8C9E1C8B3B1D683EE82F6F7D4BD3CE39FDD15F9387CF94DC5F29C31FBC0BE14A577054DF3A54CA8B7A06FA4EF2D524FC6B2DFBA8F667F4C1FC97EFFDC96FDD21ECD2E911F1E5BB8B9FAFB9FE6FAEA53CBC7A7D6BBE7BB5B5C6FF42F5EED0F52A77F9479787F9A6669A8F674FE9CE34D4FA8378C5E3BF4D4939954863E6C25CDAD3EFCDF8DB56265ED687BF7F8CA48A738FAEAD6F6FFAD9F', 2, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (1, 'http://127.0.0.1:8000/shop/category/headphones.html', X'A5514B48545118FEFF73EEDCD17B9D99EE6546C7711C75AA418616C75E42B9188286302B1F45430FB052C8A00796BB8A2BC840B5B941909048484C0F87B48D48B6B80B8B6C1121615112177268A8450369F45AC42F4D8C2E5CD4CFF9BFEFFFBEF371389C8333CA8882EE6DDD9D473A8E75F79C3CDAD475F65C1C10113932B9089171C98388CC213B11194947313272B84CA012AC2228AB2717998C0C15646E580CCBCEA262455555B564CDDA486467D3AEDD7B9A5BA85ADB4ACBFCE5818A65B0375819AAAAAEA921085585570300A4104435C04D1E01305D30AF0930914CEF1C7ABF31EFACE44D611629037000C409318322C901C2E1F07100888181602118CC9428B2032010003A106CFE462A1063C50502461D05625C85312D853100A88633002046E53FA35DB238A4D06269DF3B6FDA97455B37D162CD166BBEE58E9BAEF8358C3788B42F61EB893E141D620CC57961EBCF3D861E35F468637410A386DE94534D34F4AC6AF3677C806F1EE0161BE0B1C2957365D59C4BE45C96F250321D49695632B9C99392484A0DFFE6E45C22EDBBC36DFD051A32B530E4F645147FF55261EBB5F4BBF9EF5514450D540483959A47D469EBA9B40D1EAFA669BE65A0959639FD4E452170FA9DE5B0424D0DC59F8CB4ECF3E775A8ABEEE5E9D0EBEDF1CF4397DE57FE785B98BDEF5E68259FBA7732DA9DF7AF5B77E789D389CCF7CEC3995FD307333FBFB465BEB6B76616C8EF3D742191CFCE7D6C794CBCEEA935727970BEE77FBCE0ED928D85F7BB78A52F424CF79E7C706F9A66A5BFF614716CB4718278CBF0D5FD8FCA27A7F4FE4F9B48B3951E27BFBF0C96D6D6B60FE3C3AF6ED417F66F', 2, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/', X'85524D4814511CFFFFDF9B0F9DFD1E77D775CBDDD5403C747841269887285AAA9322A1870E4B64E4A53C281544CCC14B163441D8621121B1446EB8814850873D58ACD1A12F4CD0C3404B6D1B28C11EC44BF1771B1975A9C7FFFD3EFE1F33CCBC878BDAA407D5FEC1B37D432383270111912353EA1019977C8432229390A9C87CC890320A818BC04F1022E8A20A3205196AC8BC40E3B2A2D6D56B2E97CBE5F6787DFE802E355423180A37469AA27B6AC1DEE6583CD1D242108BB7EE0300C8208804C023DE06607AA0121060222583450CAEB3E08A14CC6009452265602ACD522526D23240341A2DB371A5CA13EE2A8B5BF5007081AA408F048B17248731EB1D66C9E33090931DE65D388306024002860140E494BFD2726F8A0C1A3C1B9A8D664325B47413F3AC27CF7A1E7B93A627791793DD221B1AB0F4813114E7C41C8A6BC2D28B41432F4879FF95BC5FE458CAD06779599EE516CFB33959D49686BEAE19FA094337B527922957F803C9E426AF7051E1DDD5CCCAAECCFF7A2C697B4F3654E296FE1E0D85B63094D4268A2DBFDD587A3B1DBF7DFE9AA6B9DC1E75F3025423D0E00B060281502D0837AA1155D308D488DA04FF580B53C93733BDA723B68F0D1DF87C29B6743CB93675E36BF3C6B29DFF7695AFFEBE37D31549DD9E70CE53BFAD69CED6344F6CF497E33BDFD9BE01FADAB1E10FA43BA2312FF1B3426B677CE4B2E2ECFBB53C7A706EE3FC5BD2C572EF6BE2FD85FCCCF8C3CAA8B3EFC721737295173F3973D76F8EB511D337CC3F7FFA91B4966EBF487C2477EA2571D7F49DFE574DF30B7AFA670779B6EBEF303BB6CAB560DB3ADCF7FDC599F0D1167B4F7FB9DFB973FF01', 3, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/product/charger-20w.html', X'A5525D6C1455143EE7DEF9D9CEB25B667687B60BDD9602655584BB4B7F28F260346E8CF1A1C4189A26980AB5910785D4561335641E6AB26AA363825A9118427063A00A26A42991870D41D3E2CB0A8A8D2D3081715D311149D664B32F9A3353C8761350E3B9F77CDF77BEDC7BE74C72F092F685CEC28F0C3FF7ECE0EEE1D117763DB9E7A511B57778EFE0E8EE91214044E4C8940022E352BD5F224719550C91254791216B4286884C42A622D39085902D47B60993BECF158220C172821D64225390A1862C0CDE438A1AA8D382C16070D9DA75EDD2FA84E4AFFBEE7F60C3831B379922994C999B3B3ABBBA6BF616B3C7DCDAD0D8145B5903AB9AE32DADAB5713C45BDAD600006411442BC061DE0E6087A0A40BB091CCA88BD1328B2E48D12C1691CE00B4B6423100C0D86104C6720C18733830361D84542A9565A9024B15786A4A12731A003C0F003613F9FA454D9FD300FA7D3A853EBFC37C2E2EF259EEB138600288532BFCBBB1185093E0F019A9AAB0EBAA8AB9505501A578162DA4A6611F008893CAA2749679228B169F3417DA26CD223A868D39D69B63BD47C3693B943E80E96D62D2EC738CBE311483620AC5EBC231C69B2DA35C57D628F3013B6007CA9A8DB4C671FCAE3CA39C975D79015D39E1CA03AE2C5C7997654C4B96F1B865E4B965FC81969143CB18933DEB90E4677F56EBFF49EDFF8F62C07BA3EC3F65CB9F49B65CE287249BDBBCC445896FBB9BB3704F67D22C72C7C8A3A5500A4B19F050DCA997168E91A011BE3DC39AA605EBF598BE7295EE2F6F0EDBF488AEEB7A64CDDA75EDEB6BB61ED123F5515DD7CD1A58D1A036AA9A46A036AA4D708F983D92FEE6C4F6A71B6FD7F13DC9EFF7C6E71E4BDF3CF2E6F5E6CA3CFC8F4854C0B894D0676F3EBAEFBB1B5BB2F99EE0A73F905FF7EA9521E23722958BAFBDFB5717E9EB85D3348777E2DBCB99F777F614AE917EE66AE683A31D85ABA46F39990FBBBB0A0EE9B16B9989FDC9C265D20D6EE6237D73E10AE9633F670EF66D2C78BDBF6CAAA789851BFEB3B6BF5BF3A31D5395A1F3D5DEAF5DF6C1DFB97B9174672C1E26FE7CA6ADBB65E41585B47B63FBD7C41B667227DEFAA4345A7D77FFDB63EDDEBF3457E6CF7D79EC02696D22F122F1C3279FF88A78EBF1F7769C693A376B4CFCD649355BDA12A3EDAF7F19DEC11A58120F3DF5CBF4F11F3FEEFEA7FC1B', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/product/charger-65w.html', X'A5536D6C5355187EDFF77EB4BDDD3AEE6DCB3ED8BA6E8CD1313E6ECB3ED8074A3434C4F863448CCB120DB2B9480C8273538386DC1F23A97389D764E21CC32C643664CC80FEC089FEE88F61864619823ADD180DD45287019754B3F48FE6F4AE4BD7347E84F7BCE77D3ECEC97BCFB9C9C11FA4530EB23DD2F5DCB31DED5D3D870E3C7EF0E56E534BD7E18E9EF6EEA38088C821896644E2F83C432287029AD08279CC14C47C24A4A48DC4239990AA9070454848B9486B90EC486B91761B6B9CC88A959535ACBCC04C24110925241B18AD4D668B64B55AAD39151B2AF98D1E3E35AA3655376DDEB2759B53F57A9B77F20F3DBCCB6BA4CFB9BDA6B6AE3E6B36381BF30B0A8BD66594E21257A9BBAC8C155769F97A00802082EA0618E12A01F45C88CB2AE8C84C47041D4BE498E31D418C21DB03E07643CC0C403482401422200A7340A409403461059FCF17245F8C7C41CE17E47D7FF2EA8C0400CF03804EA0F1E001800007E0F6BB47B14F34F0448E81EAB07379333B9704D06AC02768603F19185DC61067E0106F60DC4075A0D868535404EC6210E6A6F834A15BD2C44C6E9A80EB1541D4905D148E00807A5E5CA6E19C2409A2C68D3B2F7AC69D310C2B3A86A825442DA336BF9EEB1F407FB33AEE6C0D2BADBDA876A817507D430D2BBF956BCA9265496273DAAC9B75F392A4231BFDD8FFAF3825F68911610E2382679F1A11F6470435221CD0945E4953F668CAB045537E474D09A1A60C0B9A12A2504E883465801684011A13768C096A36A6291362B2C1B460CCB6A0D4F6B3A9ED81C992906CAB0B67785D887373BCCEE95C9C53E35CB3E10CFF4F67DC19E3C2CA346A229BAA26EE4F567545AF1661C5C39E53EA3D499264CD938BE475C5726A94B84A3DEEB272D92ECB72952CE76DAA968D94EDEB2B36546ECC9AB23DCF21CBB233A3ACCD3715982489155381A910FE212E9FF67F796EEF930529ED3AE8BD7ED835B3DB7FFFF49BB74B12B3F080515BE4B27D34555E1FB8F8ED2BF34DCDDED2EE57C5851DC1E906EB87DFB375CBD1F94E86C7ED896BAFBFFD575DB61E573E6D3C6E6DB8B4C8F8E64E6DCFE4BD970E31FE5AE3BE2DDF5C6D1E61FC76F433F63E56E2EB1B81779F6E88DE62FC999B8113A335D19B8C2F8603EFD5D745C38CF7DE0A0C1EF3466F309E1F09BC2F6F8FCE333EF64B60A8756B3479F79D770227EF6E8BCE313E7225D0EFAB8EFEC4B81AB1FD91FA566461EFA5E4D9A642E7FA3E88F7A49FC39300E5FEA347AEA67BBFD6E943F7B8C8B5746F71B6A7E642A2F3AB74EFD85BBD95C9BB952466273F1EFB8E7169D0F322C35DE71FFB9C61E3D9779EFAA270F2B23278B796695AFDEB88656A64AEFCE748EECD28ABA2E9893B13677F3C599F6DF69D691733BDBF01', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/product/powerbank-10k.html', X'9D525D681457143EE7DE99D9CD8CD938935D936C92CD1A35AE3FD59B353FC64A291517297D88882801FFADE88356D4205491794861DB4A3B2DB6A65A8A885D44232A48883F0F8BA824FAB2FE07137588C37655F007B6B0EC4BE5ECA86C5250F19C7BBEEF3B1FF7CE9C878377D4D3E3D1F7C5B6AFD7AC5FB7AD73F3DAAF366DDFB10A10113932C58BC8B854E6B6C851468D0CD94086AC02192232099907998AAC14D974D7E20A8146309EA09D4C640A325491F9A0F00DC5E32D51354DD3C64D9ED2204D8D4894D3A6CF98F9C9ACD901D1D8180DCC696A6E692D3E6D8179159555C1EA3150531BAA0B4F9C4810AAAB9F0400904010618043BC01C02A85AC2EC04232FD0EFA73CC3F2CF9139841BA03100E43C60BC0D82104C6920C18EBD3201A8D265834CDA2691EED95C4A00A001B01C0622255F65AD39F54800E97CEA0CB3F3197332E8B737EF77A30083412D8BC5F2A6AAC92A266B0B4A8817C4D024DA411612B008853CA6B698F2B88049ABC273012EE0964D0362C4CB2F6246B3FE28B59A5B17D189B2F7A02CB6D6379178AF5A217C56E611BBF569B46AE24A752A5BC96D7F2E6540B29F7E2DEFF71BF725576E46174E48823AF7664E1C86B4DA34F328D45A691E2A6F1024D2389A671CCB52E72B73A126AC73D4FC787899C5C786BC947254BCEF23F258B5B3CCB4596CF779DE10F708A5FF50432DC3652682A54C254561750BCED4737B611A1957CB393AAAA6A657A50AFAED1290BCB55AF97EBBAAE974F9A3CA5616AF1D1CBCBFCBAAE07C6C0840A4FA54755093C959E2A78470C1C8E5D39B97869E59B3EB4A9F1D637A1C185B1E787BF7F549B1F828F88C72DD681E181CFF63FE3CECD277313A936EDEFDBE4977CFB6003F177E5F99BBB7EFEAF85F4A3F459DAB0B771ED7EFCB7156DE911D22B1FC67F3FD2947E48FAA51DDFDFDA92B649778DC4BBF734A6EF93AE70E27FE873D20F480BC7F7EFD8595E0E7536F5E6375C25ED3C597C9978667FF2E40F7F653B493707433EE213FDF5AD753B762AC56F2379309E2FD87ABDD8DBF363574361EEDAFCD0A5D3C76E9056BB235B883F3FF5E579E279C77F5976A1EAD280D1FDB4997A367A244687F2FD51B8330646C5A74BFEE93B7EF760EBBBEA15', 1, 'layout_without_markup');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/product/cable-2m.html', X'95525F685367143FE7BBFF921B9BF6DEE4CE366DD33FDA363A755F63FF58E7C3D830C898501932297354ED643E68A5B613DC90FB5021DB64BB834D8B1B43C405D10EDD1852D1873C3889F3254E37CB5AF5A2979829CC091984BC4CCE8D8634CE82E77EE7F7E770BEEF9E87837FA8D76B98FFF591F7B70C6D1B19DBB9F5AD1D7B4695FE91E1A1B16DA3EF0222A2804CF6203241AC2E5A14D04B56AA4186CC4086884C44A6205391B56147B122C8043E821A82F5544426234315991FDC2764C5E3557D3E9F6FC1E2B676B1232246C4254B5F5EB67CC52B06EFEC8C1A2BBBBA7B7A4B6795D167AC5E585B17AAAF8086C67053734B0B41B8A975110040028137031C15DA01AC2AC8691C2CA462D0C1609E0567C56002B3483D00CDCD90F50030761481B1291F44A3D1298CE6319A67D194C0A75500F800002CC6D3D54F34FD44051828D2CF58E4CF99CBDC0E0070D328F6864240A3802DA4C4326379CBCC745599816463024DA4D1603700F033F213692F7045024D61D238D1326964D1D62D4CB2FE24EB3FEE8F5955B1AF30B6864F1A9B6C7DD338F2217E16F9C7DCD6A7EB4D3DEFCDAB94698FE5B13C79D542FA0EE2C112A7E459D191763952C491061D893BD256539F124D7D9DA9A70553FF074D3D8945BDCED4B3AC98030975E04F65605E31E8DEC84B2E59D209D19272C2B7A22558424EE039614DB132FB4C65FE9E49232BD87A1A4D99929BF2A08BBCE4E71A5B8FD0EE3D5D3E55557DD55A48AB6FD01A34778D5AB580A6695A60D1E2B6F68ED2D1025AA03AA8699A51012F2D546A155525506A953A98272E1F8B5D3ABD6163ED531FDED1797D383CBD36F6F0D827771B0B33F002F16866AC6B387EE1A7B385EDBFDE5F9548F7F9BEFF9DEADE7DB7B6131F0814AE7DF4C57F3DA4EF66CED12A95E2CACDF8D79BFB327748BF773B7EE87857E6B6FBA61D3FDCDB93B1498FDF894FECEFCCDC24FDA1A19C23E68EFFDFCA392205D01FBEB1FB2AE9EE50D84FFC43AAB5B76974AF5CDEF7578F75E46FC1B946DAB9BFE117E265A9E4E94FBFCB8D95F7EDFF6CBCDD9DB9B13073F1C793BF91562722BB885F3BF3E679E2D5A7BE7CE742DDC5CBFAC4836EF26CEE488C4E45EDFFC26DA98039F1EADBF7A64EDDF8A6F779F918', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/category/cables.html', X'CD925D48145114C7CF997B77D69D71779D615777375D750BB1E8E30656503E98E566166149248460A9910F5958BE454C21621F0F130489ED43486C1F2E683D2C513D6C2081D2434894640F03B5B4D0431B59F4F510674B99EDA17CF430F777EFEFCFE10C335C7CA13CF6A0A7BEB7EB5067476FDFB1C3BBBB4F9E6A03444486925C802831EE4564DC812891385C2821C53241251411FC8410A19CB082B09220F6503B4A324AA8A0E481DC14D959E0525455550BB7F2FA6DDB1BA23B1AA9763615970482A1658BC5AED2B27079456525215C1E590E001047101500D7591580E986394D808914FADEA2EFABE47BCD7D71CC20F5884B5C0C3BC4940C1089448E02401D180829044332390D6B04088580A681C55E719B245D3601C3616FB35BD265373094BC469B255D7683192F0C07E368200054C0090010E3F29FA355983BC4D16009FFB9B2843F83966E624A6A4E49CD373C51D31DBD82D15A91F0B75A7A6B3F8A4E9144715A58FABD12437FCA7EAF9EAC6AA2A10F16596C8AC5D8C6184B493156677FB2EEC1A2AC5B64DD96E701B7F8008F7193996C808B015EBBD492AC5B24FC1966E9CFD0906909436ECF512C78BE587A355DC7F9FBA8288ABA5A0B6BDE356BB575549AF0FA344DF32F165A718933E054148233E00CC23F6A7224FA646CEFFEC0BC87BBD73F3F1E9E69887E1839FFA6ECFBACBD97F2B313AB7AE7FD6AEAF61CED89D6F4B7AEB6F4CFE983E91F9F5AD25FDAF7A53F535E379619FEFB7D1F67FB6A92DF8F4C2DF5ACF466618D3D3F73B1BF8A76FA271377EF4CD35919AAEEC97DE778D343DA378F5E3EF0283831A90FBDDF402ED907E4C942B058E4D7969677F72FDCEA90475F5EDBF4BFF50B', 2, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/info/about.html', X'63BCC6D5C0C6C8E754949A98925C549A9BE493595CA2CAC0C8C8C8C8CCC8C4C6C1C8C8C4CCC2CFC8C4C8C4C826C6C80862B0313231723132F13180A458D9D83938B9B8B9B9B9797C7CFDFC0D036464E5E415141541849CBC9232030303C33246060305068605CC6A0C0C137819BE081A304C6004098A3C6114F9C1247287456419E30B46030583658C0C0C1FD81918140C8EB1304CE358C6D8C0C8C0C0A0C050C0C0C060B0810DCA7CC003662C636C605E23FA8F7B8DE80BC6074213180F30051C600A58C2E73681D76D0AA39B8DC11AD1880742112D8C062906DB180D6A0C1E08AD636B10CA6B107AC07480FD00E304AE074C13B81630350879AC117DC1FC40E80263031B081B34B025804903381F95F3404803E47998EFB9B8B8B8E5F80505050DC5C4D925D8B9B84004BB04BB24C8F3B8C0C9456EC7D607864AC0F872998657F2E56EB8BABD5FD4F958F6D76D9018D70C8D3C643DD5FDFFCD76983FB9822C56D7DDA206A2417A8E6C5C7509599FC306AFDD20DA6AF5C4F03D92474E0ACD786D0AE233A1B804950706D6C1CF7774AD48660300', 4, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/info/returns.html', X'63BCC6358D9D91CFA928353125B9A83437C927B3B8449781919191919991898D8391918999859F91899191898591899D91898791498E9191918991898D9189918B91898F01A484958D9D83938B9B9B9B9BC7273028D8108242420D6564E5E415141541849CBC9232030303C33246060305068605CC6A0C0C137819BE081A304C6004098A3C6114F9C1247287456419E30B46030583658C0C0C1A1A1A3318193A9820F404160D0D8D698C0667D819181802188E702D636C6064606050602860606030D8C006653EE0013396313630AF115DC3B746F405E303A1098C0798020E30052CE1739BC0EB3685D1CDC6608D68C403A18816468314836D8C0635060F84EE703408E535087D636D107AC17843D0E386A001761614AD117DC1FC40E80263031B081B34B025804903381F95F34048031460B010E3E2E2E296E3371216842041634131717609762E2E10C12EC12E090A305CE0E422B763EB034325607CB94CC32BF972375CDDDE2FEA7C2CFBEB3648ECC8C6559760F2D209D7FD4E682E3605B17B3B2E7997713D9705B17FAC52E6B9FAEEE66A10BBD47997ADAAFBFA33C8F6D475B7A88168909930F3B86668E48168870D5EBB41B4D5EA89E17B248F9C149AF11A6C3E133E872303EBE0E73B00', 5, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/category/headphones.html', X'A5515F48535118FFBE73EEEEDCBDCE752F9B9BA6732E1089FE9CE81F940F62B9CC7CD0241A5460A5900F59AC7CABB8820C2A881B04094984C4A21CB95E44B287FBB0227B0A898A92B890A3510F0D5AD2BF87F8468BB9071FEAE39CDFBFF3F17138075F2AB30A56B5C5068EF61F8F0D9F3CD63578E66C1B20227264720522E39207194764241D2E6448A14CA012AC22F06FA614998C0C1564555068969D152E455555B5B26DD7EEF6C89E0EAABD9DD5FE404DEDEA32D857571F6C08353612041BC26B000012082204708B3701986EC86B024CA4D0BB88DE6FCCBB20791398451112F751BC6300E170F80400B48281602118CC94684E07406D2DD020B0F91BA9C44CBB4A0CA41C256646852B5A020D0480109C06009192FF48BBB2201268F0A46FC99BF465D1D64DB458B7C5BA6F57454C77E41A465A44D217B5F5E8288A7E318DE29CB0F584C7D08772AA89869E556DFE8C8FF36DE3DC62E3BCB574E5DC5935E71639B7A53C944C475C5A904C6EF2B824E252CBBF2539B748FAB2DCD69FA321D31686DC5740F1D72F37B6DE4CFF58FC484551D4A0E659BF41DB48A5098F57D3345F1968D57E67C0A92804CE80B30656A8B989C893A99E0381A20F0E6E7A712AF8BA3DF279E2E2FBFA1F6F29233D925E1B2BF65CB7EEE68993D1CCF78123995FF387323FBFF46696FAF667BE523E72F87CB4D8BBF8B1E731F1BAA7D6D4A59BF9E1FFC9EAEE546E29BDFB85CBA34DC474CFF4837BF3A495B1E621E2D654E72CF18EC9AB071FD5A4E7F4B14F5BC9B315DEA2705606CB6B67EF8799C95737B697EEDF', 2, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/product/headphones-anc.html', X'AD525D6C1455143EE7CEDF76966E99D95DBA5DD89FB65056A930BBFD594B6B62306E0CF1A1448D4D130C422592A890DAC6440D99879A8C587134A21588691ADC9852032486D4E8C3C68029F8E022FE545ACAA60CC30A8948B292CDBE68CE4E21DB1A88249E73CFF77DE7BB77EEDC9B5CFC45BEA230CFA6FE179EEFDBD13FF8F2F62777BD3A2075F7EFEE1BDC31F012202272C8441722E3F81AA7450E05749321885E64C802C81091F1C8246432B26A64CB917991AD4056872CE94C7322819B6039C156329189C85046E6016743C95525BBDD6EF7B2D56B9AF8B5319EF28107D775363FB47E835F8BC713FE96D6B6F6E4DD46877F636DA02EB87209AC0A8523D1FA7A8270A4A11100208DA0450146B92600B31A0A8A062692E9B3D05764BE59DE97C63CD21A806814F22E00C6461118CB30606CD20D894422CD127996487389349FB8C56BD33200BC080026D3B2350B9A7E2603F43AF4253ABC9F399C5FE06F39870FF30EDF5AE0AC50666DACD6D92E18043A35E4B829BEA231AB2A9AE9EA8A06E62369D4916E017B00403B2E2EC8DCB2B248A3CE4DF8CF364EF8F398534DCCB0EE0CEB3EE24999D5A90F31D5A54DF87B726ACF106A7DDA49D4DED4726A29A4ABC5AAA24C9575992ED355944DA41CC6E17FF194B84FB48459B484D8D39A256CB304CD12B6EBEA24AFAB4FE86A96D3D53F515733A8AB1F48656B5C74AA372DF75E907AFF3F5114CAFB9BC2E7BC2914B8C3BCC9995C81D30A5CD77F77662B9C097F9ECBA959D4452A4D17B79551BBD32F6E726A8C1EFAED972ECBB2BB46092A2B572994A1702416AD6F50BC8AA228DEC6D56B9AD6DE6D28DE1A9FA228FE25B0A2560A48B24C2005A43AB8479C194B7D776CCB3381DB7D7857FCA7DDE1E9C75337C6DEBE1C2ACDC07D86756DCBE9E6A9CCB103BEFDE65C67577CDFA785C16B0FA7B31DEECF7EA6F9AAD7E77612BFE52D9D7FE3BDBFDB495FB6BFA2877927BEBF681CD8DA61CF937EEE92F1D19156FB12E99B39E3E364BB9D233D346F8CEC8DDB1749D75AC6274A8B3D477AFC8A71B067BD5D3EF723578D43D737D8B3A4477F308613EBECDF48B79C33DE3DDD6C5F20AD599EBF96DE215602F5C6637BCE557A3767065B4F96769EADF47E6F370FFEC159E749B705C31EE22FA61A929181D7C4CA757BDF196A2ADF33549A397562FC47D2F248EC15E2478F6FFE9A78E3D1F79FFDA6EED41975E47A1BF56CF191180DCAFB8CF2074B6051743E7575F2E8AF8792F7AA7F00', 1, 'structured_data');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/category/chargers.html', X'C5925148536114C7CFF9BEEB9DBB739BF7B2E95C3AE70AB1A0FA8254281F447359F96049244860A5900F9998D14BC40D44885E6E14346C0F21B12857DACB888AD88344F6541616257121478B20075994F51067B5B826984F75B8DFEF9CFFFF9EEF5CEEC787D3CA7927BA1AFABB0F741DEA3F7EE4604BCFB1810E4044E4C8E47C44C62537328EC8A8CCB32343326582835048F012FC8472C21AC25A8268A1766432325490B9203B45B6E5DB1587C3E1286868DCD614DEDE4CB1636751B1AFC4BF6AA5D8555A16280F56541002E5A1D5000031041104B8CC2B010C27CCAB020C24D3338B9E2FCC33237962984611143751BC6600A150E83000D4838E9044D09921D19C6600BF1F681098FCA5641109BB45C0789EB5CDAA1276AB0253B1365A55C26E55902E84FB4531D4110082D00700625CFE559A05D922863A8F7B63FEB8378DA6666092B52659EB1557D870862F60B84EC4BDEDA6D63E88A24B24509C14A636EDD1B5DE8CC3405D8BA8267FC4A3BC26CA932CCAEBAD4FC61951334E917166DC7724236F488A4A0637F8902486A4BA9FCECC126769CFBFD995718AB837CD4DED31EA322DA1CB9D598ADF7AB130B52ABA7FB90BA8288A23A0BAD76F503752A8C2ED5155D5BB52A845C5369F4D5108369FAD049689C991F083B1DD7B7D391DE8D9F4EC68E04553786EE4CC9BB2855739EFF4C4BAFE5CCFC5E4B579CAF1F6D4D7EEFDA9EF531DA96F1FDB529F3BF7A43E915F3F961EFEF33B550BA0CD35F63DA1BADA1F7051BEF130545B3E7042B6F6BDAB31863FF0D9A7FF6B6FE9D582CD56FFD4D9C14ACA741613B7AE4F51AD44AA7AB3FF39BEF32EE52DA3E7F6DD2B9998D422EFAB49B365CE3BFB6EA5581C5BDBDEDE1E7D7EA9F66FEB07', 2, 'shape');
INSERT INTO page_layouts (crawl_id, url, layout, template_id, assigned_by) VALUES (2, 'http://127.0.0.1:8000/shop/info/shipping.html', X'63BCC635918D91CFA928353125B9A83437C927B3B8C49481919191919991898D8391918999859F91899191898591899D91898711CE5403311999D8189918B91899F818404A59D9D83938B9B8B9B9B9797CC2C2830D2128C224322A2A323A3A323A5A46564E5E41511144C8C92B29333030302C63643050606058C0ACC6C0308197E18BA001C3044690A0C81346911F4C227758449631BE6034503058C6C8C0A0E0A1B083B18B0D424FE381D006FF9841263130CC6362600880508B1821F41E8E658C0D8C0C0C0C0A0C050C0C0C061BD8A0CC073C60C632C606E635A28B78D688BE607C203481F10053C001A680257C6E1378DDA630BAD918AC118D782014D1C2689062B08DD1A0C6E081D015B606A1BC06A1154CAF585730AD62F558C56A808DB58CF91B230407CD630AEA600CC264AC117DC1FC40E80263031B081B34B025804903381F95F340480314CCB070E6E2E2E296E33711168420651555535355333355333331717609762E2E10C12EC12EC980079C5CE4766C7D60A8048C2F976978255FEE86ABDBFB459D8F657FDD0689C1681050DAFBCFF49F08BF3E88FD5D27FF49D0963FDD2076FFF3C419DFE61D99856C76D4AAAD4A11F51D1920B6CBEB4D3F824D5C1A41ECFB0D3E59BFCD626490D5D675B7A8C1EC3AB271D525109B6B86461E8876D8E0B51B445BAD9E18BE47F2C849A119AF4D417C267C1EC306AC839FEF0000', 6, 'shape');
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (1, 1, 200, 'text/html', 5272, 11, 0, 'Robin Electronics | Robin Electronics', 'Chargers, cables and headphones.', 'Charge everything', 'Popular right now', NULL, NULL, 'Indexable', 38, '3110f79e6a08d67b36c59b1c4b52e596', 1792681167, 0, 1, NULL, '["WebSite"]', 6918797436566768178, NULL, NULL, NULL, 0, 0, '[["connection","close"],["content-length","5176"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 356, 218, NULL, 8.81119003295898437e+01, NULL, 38, 'Charge everything', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 1);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (2, 1, 200, 'text/css', 5409, 40, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681167, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","text/css"],["content-length","5409"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'link', 0, 0, NULL, NULL, NULL, 2);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (3, 1, 200, 'text/css', 199, 40, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681167, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","text/css"],["content-length","199"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'link', 0, 0, NULL, NULL, NULL, 3);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (4, 1, 200, 'image/svg+xml', 219, 45, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681167, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","219"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 4);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (5, 1, 200, 'image/svg+xml', 189, 40, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681167, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","189"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 5);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (6, 1, 200, 'image/svg+xml', 212, 40, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681167, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","212"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 6);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (7, 1, 200, 'application/javascript', 1343, 40, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681167, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","application/javascript"],["content-length","1343"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'script', 0, 0, NULL, NULL, NULL, 7);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (8, 1, 200, 'image/svg+xml', 0, 15, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'N/A', NULL, NULL, 1792681167, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'other', 0, 0, NULL, NULL, NULL, 8);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (9, 1, 200, 'text/html', 3314, 12, 1, 'Returns | Robin Electronics', 'Returns at Robin Electronics.', 'Returns', 'Register the return', NULL, NULL, 'Indexable', 41, 'b8b0bd8b4163fa678e3c9d2f9d9738c9', 1792681167, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 136483128739668165, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","3218"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Pack it', 262, 181, NULL, 5.65517501831054687e+01, NULL, 41, 'Returns', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 9);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (10, 1, 200, 'text/html', 9045, 11, 1, 'USB-C Cable 2 m | Robin Electronics', 'Buy USB-C Cable 2 m from Robin Electronics.', 'USB-C Cable 2 m', 'Description', NULL, NULL, 'Indexable', 298, 'c954681d1c128e261006b288ab5ab362', 1792681168, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032579273232984, 100, 4, NULL, 0, 1, '[["connection","close"],["content-length","8926"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 356, 288, NULL, 5.01130943298339843e+01, NULL, 294, 'USB-C Cable 2 m', 0, '["http://127.0.0.1:8000/shop/product/charger-20w.html","http://127.0.0.1:8000/shop/product/headphones-anc.html","http://127.0.0.1:8000/shop/product/powerbank-10k.html","http://127.0.0.1:8000/shop/product/charger-65w.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 10);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (11, 1, 200, 'text/html', 3702, 11, 1, 'About us | Robin Electronics', 'About us at Robin Electronics.', 'About us', NULL, NULL, NULL, 'Indexable', 105, 'c678ef56f5987705c294e4bc978da2d4', 1792681168, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 8439090172742799370, 90, 1, NULL, 0, 2, '[["connection","close"],["content-length","3608"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 275, 188, NULL, 5.65517501831054687e+01, NULL, 105, 'About us', 0, '["http://127.0.0.1:8000/shop/product/charger-20w.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 11);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (12, 1, 200, 'image/svg+xml', 213, 43, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681168, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","213"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 12);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (13, 1, 200, 'text/html', 9213, 11, 1, 'USB-C Charger 20W | Robin Electronics', 'Buy USB-C Charger 20W from Robin Electronics.', 'USB-C Charger 20W', 'Description', NULL, NULL, 'Indexable', 298, '4e6dd530689561da77790d8a7634ff05', 1792681168, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032578700710472, 93, 3, NULL, 0, 1, '[["connection","close"],["content-length","9109"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 390, 307, NULL, 3.54659347534179687e+01, NULL, 298, 'USB-C Charger 20W', 0, '["http://127.0.0.1:8000/shop/product/cable-2m.html","http://127.0.0.1:8000/shop/info/about.html","http://127.0.0.1:8000/shop/product/headphones-anc.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 13);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (14, 1, 200, 'text/html', 8927, 13, 1, 'Wireless Headphones ANC | Robin Electronics', 'Buy Wireless Headphones ANC from Robin Electronics.', 'Wireless Headphones ANC', 'Description', NULL, NULL, 'Indexable', 304, '8f4006756811a585be8e7ac330b41705', 1792681168, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032579273232984, 100, 4, NULL, 0, 1, '[["connection","close"],["content-length","8829"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 452, 347, NULL, 4.06345481872558593e+01, NULL, 304, 'Wireless Headphones ANC', 0, '["http://127.0.0.1:8000/shop/product/cable-2m.html","http://127.0.0.1:8000/shop/product/charger-20w.html","http://127.0.0.1:8000/shop/product/powerbank-10k.html","http://127.0.0.1:8000/shop/product/charger-65w.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 14);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (15, 1, 200, 'text/html', 8091, 10, 1, 'Power Bank 10000 mAh | Robin Electronics', 'Buy Power Bank 10000 mAh from Robin Electronics.', 'Power Bank 10000 mAh', 'Description', NULL, NULL, 'Indexable', 295, '7ede9ca54b3df83238a202002f71a1ce', 1792681169, 0, 3, NULL, '["BreadcrumbList","ListItem"]', 8403061376294261276, 90, 2, NULL, 0, 1, '[["connection","close"],["content-length","7992"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 425, 328, NULL, 3.54659347534179687e+01, NULL, 295, 'Power Bank 10000 mAh', 0, '["http://127.0.0.1:8000/shop/product/cable-2m.html","http://127.0.0.1:8000/shop/product/headphones-anc.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 15);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (16, 1, 200, 'text/html', 7341, 11, 1, 'Chargers | Robin Electronics', 'Shop chargers at Robin Electronics.', 'Chargers', 'Filter', NULL, NULL, 'Indexable', 89, 'c8a354d74a145e70f4b3e3cac2de1aa0', 1792681169, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 8096535202084659764, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","7245"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 278, 223, NULL, 7.19831085205078125e+01, NULL, 89, 'Chargers', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 16);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (17, 1, 404, 'text/plain', 262, 1062, 1, NULL, NULL, NULL, NULL, NULL, NULL, 'Non-Indexable', NULL, NULL, 1792681169, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["connection","close"],["content-length","13"],["content-type","text/plain"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5.21818389892578125e+01, NULL, NULL, NULL, NULL, NULL, NULL, 0, NULL, 1, 0, NULL, NULL, NULL, 17);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (18, 1, 200, 'text/html', 3425, 11, 1, 'Shipping | Robin Electronics', 'Shipping at Robin Electronics.', 'Shipping', 'Home delivery', NULL, NULL, 'Indexable', 30, '6850c0f40d5607a4761c442be3edae41', 1792681169, 0, 2, NULL, '["BreadcrumbList","ListItem"]', -7201497713299298398, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","3329"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Pickup point', 271, 187, NULL, 100.0, NULL, 30, 'Shipping', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 18);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (19, 1, 200, 'text/html', 9575, 11, 1, 'GaN Charger 65W | Robin Electronics', 'Buy GaN Charger 65W from Robin Electronics.', 'GaN Charger 65W', 'Description', NULL, NULL, 'Indexable', 327, 'c20b818dfc894238df7f416c6f269bb6', 1792681169, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032578740793432, 90, 2, NULL, 0, 1, '[["connection","close"],["content-length","9474"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 368, 293, NULL, 3.54659347534179687e+01, NULL, 327, 'GaN Charger 65W', 0, '["http://127.0.0.1:8000/shop/product/cable-2m.html","http://127.0.0.1:8000/shop/product/headphones-anc.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 19);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (20, 1, 200, 'text/html', 7228, 11, 1, 'Cables | Robin Electronics', 'Shop cables at Robin Electronics.', 'Cables', 'Filter', NULL, NULL, 'Indexable', 86, '46b9ec7837547f678cbc80615b407016', 1792681169, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 3777864772301312648, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","7134"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 254, 209, NULL, 6.42022628784179687e+01, NULL, 86, 'Cables', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 20);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (21, 1, 200, 'text/html', 5731, 11, 1, 'Headphones | Robin Electronics', 'Shop headphones at Robin Electronics.', 'Headphones', 'Filter', NULL, NULL, 'Indexable', 70, '6f80df8698704fb6a1919287ae550bf3', 1792681170, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 3863131754272399462, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","5637"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 312, 246, NULL, 6.54561004638671875e+01, NULL, 70, 'Headphones', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 21);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (22, 2, 200, 'text/html', 5270, 11, 0, 'Robin Electronics | Robin Electronics', 'Chargers, cables and headphones.', 'Charge everything', 'Popular right now', NULL, NULL, 'Indexable', 38, '3110f79e6a08d67b36c59b1c4b52e596', 1792681176, 0, 1, NULL, '["WebSite"]', 6918797436566768178, NULL, NULL, NULL, 0, 0, '[["connection","close"],["content-length","5176"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 356, 218, NULL, 8.81119003295898437e+01, NULL, 38, 'Charge everything', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 22);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (23, 2, 200, 'text/css', 5409, 33, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681176, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","text/css"],["content-length","5409"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'link', 0, 0, NULL, NULL, NULL, 23);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (24, 2, 200, 'text/css', 633, 33, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681176, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","text/css"],["content-length","633"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'link', 0, 0, NULL, NULL, NULL, 24);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (25, 2, 200, 'image/svg+xml', 189, 34, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681176, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","189"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 25);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (26, 2, 200, 'image/svg+xml', 219, 34, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681176, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","219"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 26);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (27, 2, 200, 'image/svg+xml', 212, 34, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681176, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","212"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 27);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (28, 2, 200, 'application/javascript', 1343, 34, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681176, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","application/javascript"],["content-length","1343"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'script', 0, 0, NULL, NULL, NULL, 28);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (29, 2, 200, 'image/svg+xml', 0, 14, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'N/A', NULL, NULL, 1792681176, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'other', 0, 0, NULL, NULL, NULL, 29);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (30, 2, 200, 'text/html', 9230, 10, 1, 'USB-C Charger 20W | Robin Electronics', 'Buy USB-C Charger 20W from Robin Electronics.', 'USB-C Charger 20W', 'Description', NULL, NULL, 'Indexable', 302, 'b460b4d7f80d57921246e966e9c0c862', 1792681177, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032579239686664, 95, 4, NULL, 0, 0, '[["connection","close"],["content-length","9109"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 390, 307, NULL, 3.54659347534179687e+01, NULL, 298, 'USB-C Charger 20W', 0, '["http://127.0.0.1:8000/shop/product/charger-65w.html","http://127.0.0.1:8000/shop/product/powerbank-10k.html","http://127.0.0.1:8000/shop/product/cable-2m.html","http://127.0.0.1:8000/shop/product/headphones-anc.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 30);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (31, 2, 200, 'text/html', 9590, 11, 1, 'GaN Charger 65W | Robin Electronics', 'Buy GaN Charger 65W from Robin Electronics.', 'GaN Charger 65W', 'Description', NULL, NULL, 'Indexable', 331, 'f8ed1ede15107ab2c9e75c8c02ba79a0', 1792681177, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032578740564056, 92, 4, NULL, 0, 0, '[["connection","close"],["content-length","9474"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 368, 293, NULL, 3.54659347534179687e+01, NULL, 327, 'GaN Charger 65W', 0, '["http://127.0.0.1:8000/shop/product/charger-20w.html","http://127.0.0.1:8000/shop/product/powerbank-10k.html","http://127.0.0.1:8000/shop/product/cable-2m.html","http://127.0.0.1:8000/shop/product/headphones-anc.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 31);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (32, 2, 200, 'image/svg+xml', 213, 53, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Indexable', NULL, NULL, 1792681177, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["content-type","image/svg+xml"],["content-length","213"],["connection","close"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 'img', 0, 0, NULL, NULL, NULL, 32);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (33, 2, 200, 'text/html', 8109, 11, 1, 'Power Bank 10000 mAh | Robin Electronics', 'Buy Power Bank 10000 mAh from Robin Electronics.', 'Power Bank 10000 mAh', 'Description', NULL, NULL, 'Indexable', 299, '9b77d0780e615ed4d288c2004c2797af', 1792681177, 0, 3, NULL, '["BreadcrumbList","ListItem"]', 8367032579239645208, 95, 5, NULL, 0, 0, '[["connection","close"],["content-length","7992"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 425, 328, NULL, 3.54659347534179687e+01, NULL, 295, 'Power Bank 10000 mAh', 0, '["http://127.0.0.1:8000/shop/product/charger-20w.html","http://127.0.0.1:8000/shop/product/charger-65w.html","http://127.0.0.1:8000/shop/product/cable-2m.html","http://127.0.0.1:8000/shop/info/about.html","http://127.0.0.1:8000/shop/product/headphones-anc.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 33);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (34, 2, 200, 'text/html', 9030, 10, 1, 'USB-C Cable 2 m | Robin Electronics', 'Buy USB-C Cable 2 m from Robin Electronics.', 'USB-C Cable 2 m', 'Description', NULL, NULL, 'Indexable', 294, '43a782887374e3edb03b3ee2db762ee3', 1792681177, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032579273208344, 96, 4, NULL, 0, 0, '[["connection","close"],["content-length","8926"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 356, 288, NULL, 5.01130943298339843e+01, NULL, 294, 'USB-C Cable 2 m', 0, '["http://127.0.0.1:8000/shop/product/charger-20w.html","http://127.0.0.1:8000/shop/product/charger-65w.html","http://127.0.0.1:8000/shop/product/powerbank-10k.html","http://127.0.0.1:8000/shop/product/headphones-anc.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 34);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (35, 2, 200, 'text/html', 7230, 11, 1, 'Cables | Robin Electronics', 'Shop cables at Robin Electronics.', 'Cables', 'Filter', NULL, NULL, 'Indexable', 86, '46b9ec7837547f678cbc80615b407016', 1792681177, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 3777864772301312648, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","7134"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 254, 209, NULL, 6.42022628784179687e+01, NULL, 86, 'Cables', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 35);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (36, 2, 200, 'text/html', 3702, 11, 1, 'About us | Robin Electronics', 'About us at Robin Electronics.', 'About us', NULL, NULL, NULL, 'Indexable', 105, 'c678ef56f5987705c294e4bc978da2d4', 1792681178, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 8439090172742799370, 90, 1, NULL, 0, 1, '[["connection","close"],["content-length","3608"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 275, 188, NULL, 5.65517501831054687e+01, NULL, 105, 'About us', 0, '["http://127.0.0.1:8000/shop/product/powerbank-10k.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 36);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (37, 2, 200, 'text/html', 3312, 10, 1, 'Returns | Robin Electronics', 'Returns at Robin Electronics.', 'Returns', 'Register the return', NULL, NULL, 'Indexable', 41, 'b8b0bd8b4163fa678e3c9d2f9d9738c9', 1792681178, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 136483128739668165, NULL, NULL, NULL, 0, 0, '[["connection","close"],["content-length","3218"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Pack it', 262, 181, NULL, 5.65517501831054687e+01, NULL, 41, 'Returns', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 37);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (38, 2, 200, 'text/html', 5731, 11, 1, 'Headphones | Robin Electronics', 'Shop headphones at Robin Electronics.', 'Headphones', 'Filter', NULL, NULL, 'Indexable', 70, '6f80df8698704fb6a1919287ae550bf3', 1792681178, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 3863131754272399462, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","5637"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 312, 246, NULL, 6.54561004638671875e+01, NULL, 70, 'Headphones', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 38);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (39, 2, 200, 'text/html', 8930, 10, 1, 'Wireless Headphones ANC | Robin Electronics', 'Buy Wireless Headphones ANC from Robin Electronics.', 'Wireless Headphones ANC', 'Description', NULL, NULL, 'Indexable', 304, 'c4f5748a2b449ab6e613ca8339cf5229', 1792681178, 0, 8, NULL, '["BreadcrumbList","ListItem","Product"]', 8367032579273232920, 96, 4, NULL, 0, 0, '[["connection","close"],["content-length","8829"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Specifications', 452, 347, NULL, 4.06345481872558593e+01, NULL, 304, 'Wireless Headphones ANC', 0, '["http://127.0.0.1:8000/shop/product/charger-20w.html","http://127.0.0.1:8000/shop/product/charger-65w.html","http://127.0.0.1:8000/shop/product/powerbank-10k.html","http://127.0.0.1:8000/shop/product/cable-2m.html"]', NULL, 0, NULL, 1, 0, NULL, 1, 0, 39);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (40, 2, 200, 'text/html', 7339, 11, 1, 'Chargers | Robin Electronics', 'Shop chargers at Robin Electronics.', 'Chargers', 'Filter', NULL, NULL, 'Indexable', 89, 'c8a354d74a145e70f4b3e3cac2de1aa0', 1792681178, 0, 2, NULL, '["BreadcrumbList","ListItem"]', 8096535202084659764, NULL, NULL, NULL, 0, 1, '[["connection","close"],["content-length","7245"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, NULL, 278, 223, NULL, 7.19831085205078125e+01, NULL, 89, 'Chargers', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 40);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (41, 2, 200, 'text/html', 3425, 11, 1, 'Shipping | Robin Electronics', 'Shipping at Robin Electronics.', 'Shipping', 'Home delivery', NULL, NULL, 'Indexable', 30, '6850c0f40d5607a4761c442be3edae41', 1792681179, 0, 2, NULL, '["BreadcrumbList","ListItem"]', -7201497713299298398, NULL, NULL, NULL, 0, 0, '[["connection","close"],["content-length","3329"],["content-type","text/html; charset=utf-8"]]', NULL, NULL, NULL, NULL, 'Pickup point', 271, 187, NULL, 100.0, NULL, 30, 'Shipping', 0, NULL, NULL, 0, NULL, 1, 0, NULL, 1, 0, 41);
INSERT INTO pages (id, crawl_id, status, content_type, size_bytes, response_time_ms, depth, title, meta_description, h1, h2, canonical, robots, indexability, word_count, hash, crawled_at, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, simhash, closest_similarity, near_duplicate_count, og_type, a11y_errors, a11y_warnings, headers_json, redirect_url, title_2, meta_description_2, h1_2, h2_2, title_pixel_width, meta_description_pixel_width, redirect_status, link_score, hreflang_issues_json, ssr_word_count, ssr_h1, ssr_content_missing, near_duplicate_urls_json, blocked_by_robots, is_resource, resource_initiator, is_page, has_mixed_content, hreflang_sources_json, has_body_tag, h2_non_sequential, url_id) VALUES (42, 2, 404, 'text/plain', 260, 1099, 1, NULL, NULL, NULL, NULL, NULL, NULL, 'Non-Indexable', NULL, NULL, 1792681179, 0, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, '[["connection","close"],["content-length","13"],["content-type","text/plain"]]', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5.21818389892578125e+01, NULL, NULL, NULL, NULL, NULL, NULL, 0, NULL, 1, 0, NULL, NULL, NULL, 42);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (1, 1, 1, 104, 0.0, 11, NULL, 104);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (2, 1, 9, 112, 0.0, 12, NULL, 100);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (3, 1, 10, 92, 0.0, 11, NULL, 92);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (4, 1, 11, 84, 0.0, 11, NULL, 84);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (5, 1, 13, 124, 0.0, 11, NULL, 124);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (6, 1, 14, 96, 0.0, 13, NULL, 96);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (7, 1, 15, 68, 0.0, 10, NULL, 68);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (8, 1, 16, 196, 0.0, 11, NULL, 196);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (9, 1, 18, 184, 0.0, 11, NULL, 184);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (10, 1, 19, 192, 0.0, 11, NULL, 192);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (11, 1, 20, 72, 0.0, 11, NULL, 72);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (12, 1, 21, 56, 0.0, 11, NULL, 56);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (13, 2, 22, 76, 0.0, 11, NULL, 76);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (14, 2, 30, 104, 0.0, 10, NULL, 104);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (15, 2, 31, 96, 0.0, 11, NULL, 96);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (16, 2, 33, 116, 0.0, 11, NULL, 116);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (17, 2, 34, 132, 0.0, 10, NULL, 132);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (18, 2, 35, 144, 0.0, 11, NULL, 144);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (19, 2, 36, 60, 0.0, 11, NULL, 60);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (20, 2, 37, 88, 0.0, 10, NULL, 88);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (21, 2, 38, 84, 0.0, 11, NULL, 84);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (22, 2, 39, 104, 0.0, 10, NULL, 104);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (23, 2, 40, 68, 0.0, 11, NULL, 68);
INSERT INTO performance (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms) VALUES (24, 2, 41, 116, 0.0, 11, NULL, 116);
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (1, 1, 1, 'warning', 'WebSite', 'relative-url:url', 'Property ''url'' of WebSite is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (2, 1, 9, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (3, 1, 9, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/info/returns.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (4, 1, 10, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (5, 1, 10, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/cables.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (6, 1, 10, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/cable-2m.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (7, 1, 10, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (8, 1, 10, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (9, 1, 10, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (10, 1, 10, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (11, 1, 10, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (12, 1, 11, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (13, 1, 11, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/info/about.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (14, 1, 13, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (15, 1, 13, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (16, 1, 13, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/charger-20w.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (17, 1, 13, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (18, 1, 13, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (19, 1, 13, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (20, 1, 13, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (21, 1, 13, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (22, 1, 14, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (23, 1, 14, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/headphones.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (24, 1, 14, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/headphones-anc.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (25, 1, 14, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (26, 1, 14, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (27, 1, 14, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (28, 1, 14, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (29, 1, 14, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (30, 1, 15, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (31, 1, 15, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (32, 1, 15, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/powerbank-10k.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (33, 1, 16, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (34, 1, 16, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (35, 1, 18, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (36, 1, 18, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/info/shipping.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (37, 1, 19, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (38, 1, 19, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (39, 1, 19, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/charger-65w.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (40, 1, 19, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (41, 1, 19, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (42, 1, 19, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (43, 1, 19, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (44, 1, 19, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (45, 1, 20, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (46, 1, 20, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/cables.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (47, 1, 21, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (48, 1, 21, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/headphones.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (49, 2, 22, 'warning', 'WebSite', 'relative-url:url', 'Property ''url'' of WebSite is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (50, 2, 30, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (51, 2, 30, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (52, 2, 30, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/charger-20w.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (53, 2, 30, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (54, 2, 30, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (55, 2, 30, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (56, 2, 30, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (57, 2, 30, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (58, 2, 31, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (59, 2, 31, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (60, 2, 31, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/charger-65w.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (61, 2, 31, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (62, 2, 31, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (63, 2, 31, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (64, 2, 31, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (65, 2, 31, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (66, 2, 33, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (67, 2, 33, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (68, 2, 33, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/powerbank-10k.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (69, 2, 34, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (70, 2, 34, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/cables.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (71, 2, 34, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/cable-2m.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (72, 2, 34, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (73, 2, 34, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (74, 2, 34, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (75, 2, 34, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (76, 2, 34, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (77, 2, 35, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (78, 2, 35, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/cables.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (79, 2, 36, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (80, 2, 36, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/info/about.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (81, 2, 37, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (82, 2, 37, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/info/returns.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (83, 2, 38, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (84, 2, 38, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/headphones.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (85, 2, 39, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (86, 2, 39, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/headphones.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (87, 2, 39, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/product/headphones-anc.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (88, 2, 39, 'warning', 'Product', 'relative-url:image', 'Property ''image'' of Product is the relative URL ''/shop/img/product.svg''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (89, 2, 39, 'warning', 'Product', 'missing-recommended:description', 'Recommended property ''description'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (90, 2, 39, 'warning', 'Product', 'missing-recommended:brand', 'Recommended property ''brand'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (91, 2, 39, 'warning', 'Product', 'missing-recommended:review', 'Recommended property ''review'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (92, 2, 39, 'warning', 'Product', 'missing-recommended:aggregateRating', 'Recommended property ''aggregateRating'' is missing from Product');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (93, 2, 40, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (94, 2, 40, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/category/chargers.html''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (95, 2, 41, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/''; Google expects an absolute URL here');
INSERT INTO sd_issues (id, crawl_id, page_url_id, severity, type_name, code, message) VALUES (96, 2, 41, 'warning', 'ListItem', 'relative-url:item', 'Property ''item'' of ListItem is the relative URL ''/shop/info/shipping.html''; Google expects an absolute URL here');
INSERT INTO site_icons (host, icon_png, fetched_at) VALUES ('127.0.0.1:8000', X'89504E470D0A1A0A0000000D4948445200000020000000200806000000737A7AF400000211494441547801EDC003A0245996C6F1FF77EE8DC8CCA7724B63AE6DDBB66DDBB66DDBB66D698C9E964AAF9E323322EEF976B76A7AA6873B6BD5AF6E3EF64DCD7F1F82FF5E04FFBD08FE7B11FCF722F8EF45F0DF8BCABFD2AB3D68C93BBDE41E2F7EED8A17BF760DC0DFDF3BE3EFEF9DF3237FBBC31F3C63C1BF02DA7CEC9B9A17C18B5FBBE6F3DEE02CAFF6A0235E983F78C6069FF16B67F8FB7B67BC08D0E663DFD4FC0B5EFCDA353FF5EE77706CDE78515C5A15DEE6FB6FE2EFEF9DF12F20F817BCF8B56B7EEADDEFE0D8BCF1A23A366FFCD4BBDFC18B5FBBE65F40F02FF8BC3738CBB179E35FEBD8BCF1796F70967F01C10BF16A0F5AF26A0F3AE2DFEAD51E74C4AB3D68C90B41F042BCF34B5EE2DFEB9D5FF2122F049517E2C5AE5DF3EFF562D7AE7921A8BC102F7EED9A7FAF17BF76CD0B41F0DF8BE085F8877B67FC7BFDC3BD335E088217E2EFEF9DF1EFF5F7F7CE7821085E881FFEDB63FC7BFDF0DF1EE385207821FEE0190BFEF0B60DFEADFEF0B60DFEE0190B5E08827FC1A7FFEA19F6D6857FADBD75E1D37FF50CFF02827FC1DFDF3BE3ADBFEF26F6D68517D5DEBAF0D6DF77137F7FEF8C7F01C18BE0EFEF9DF1D6DF77137F78DB06FF923FBC6D83B7FEBE9BF8FB7B67BC08D0E663DFD4FC2BBCDA8396BCF34B5EE2C5AF5DF362D7AE01F8877B67FCFDBD337EF86F8FF107CF58F0AF80361FFBA6E6BF0FC17F2F82FF5E04FFBD08FE7B11FCF722F8EFC53F02513D76C96B82D7DD0000000049454E44AE426082', 1792681174);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (1, 1, 1, 'json-ld', 'WebSite', '{
  "@context": "https://schema.org",
  "@type": "WebSite",
  "name": "Robin Electronics",
  "url": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (2, 1, 9, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Returns",
      "item": "/shop/info/returns.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (3, 1, 9, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (4, 1, 9, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Returns",
  "item": "/shop/info/returns.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (5, 1, 10, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Cables",
      "item": "/shop/category/cables.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "USB-C Cable 2 m",
      "item": "/shop/product/cable-2m.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (6, 1, 10, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (7, 1, 10, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Cables",
  "item": "/shop/category/cables.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (8, 1, 10, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "USB-C Cable 2 m",
  "item": "/shop/product/cable-2m.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (9, 1, 10, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "USB-C Cable 2 m",
  "sku": "CABLE-2M",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "129",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (10, 1, 11, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "About us",
      "item": "/shop/info/about.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (11, 1, 11, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (12, 1, 11, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "About us",
  "item": "/shop/info/about.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (13, 1, 13, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "USB-C Charger 20W",
      "item": "/shop/product/charger-20w.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (14, 1, 13, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (15, 1, 13, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (16, 1, 13, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "USB-C Charger 20W",
  "item": "/shop/product/charger-20w.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (17, 1, 13, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "USB-C Charger 20W",
  "sku": "CHARGER-20W",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "199",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (18, 1, 14, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Headphones",
      "item": "/shop/category/headphones.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "Wireless Headphones ANC",
      "item": "/shop/product/headphones-anc.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (19, 1, 14, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (20, 1, 14, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Headphones",
  "item": "/shop/category/headphones.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (21, 1, 14, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "Wireless Headphones ANC",
  "item": "/shop/product/headphones-anc.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (22, 1, 14, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "Wireless Headphones ANC",
  "sku": "HEADPHONES-ANC",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "990",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (23, 1, 15, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "Power Bank 10000 mAh",
      "item": "/shop/product/powerbank-10k.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (24, 1, 15, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (25, 1, 15, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (26, 1, 15, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "Power Bank 10000 mAh",
  "item": "/shop/product/powerbank-10k.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (27, 1, 16, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (28, 1, 16, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (29, 1, 16, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (30, 1, 18, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Shipping",
      "item": "/shop/info/shipping.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (31, 1, 18, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (32, 1, 18, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Shipping",
  "item": "/shop/info/shipping.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (33, 1, 19, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "GaN Charger 65W",
      "item": "/shop/product/charger-65w.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (34, 1, 19, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (35, 1, 19, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (36, 1, 19, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "GaN Charger 65W",
  "item": "/shop/product/charger-65w.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (37, 1, 19, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "GaN Charger 65W",
  "sku": "CHARGER-65W",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "379",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (38, 1, 20, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Cables",
      "item": "/shop/category/cables.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (39, 1, 20, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (40, 1, 20, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Cables",
  "item": "/shop/category/cables.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (41, 1, 21, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Headphones",
      "item": "/shop/category/headphones.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (42, 1, 21, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (43, 1, 21, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Headphones",
  "item": "/shop/category/headphones.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (44, 2, 22, 'json-ld', 'WebSite', '{
  "@context": "https://schema.org",
  "@type": "WebSite",
  "name": "Robin Electronics",
  "url": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (45, 2, 30, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "USB-C Charger 20W",
      "item": "/shop/product/charger-20w.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (46, 2, 30, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (47, 2, 30, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (48, 2, 30, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "USB-C Charger 20W",
  "item": "/shop/product/charger-20w.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (49, 2, 30, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "USB-C Charger 20W",
  "sku": "CHARGER-20W",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "199",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (50, 2, 31, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "GaN Charger 65W",
      "item": "/shop/product/charger-65w.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (51, 2, 31, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (52, 2, 31, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (53, 2, 31, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "GaN Charger 65W",
  "item": "/shop/product/charger-65w.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (54, 2, 31, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "GaN Charger 65W",
  "sku": "CHARGER-65W",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "379",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (55, 2, 33, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "Power Bank 10000 mAh",
      "item": "/shop/product/powerbank-10k.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (56, 2, 33, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (57, 2, 33, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (58, 2, 33, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "Power Bank 10000 mAh",
  "item": "/shop/product/powerbank-10k.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (59, 2, 34, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Cables",
      "item": "/shop/category/cables.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "USB-C Cable 2 m",
      "item": "/shop/product/cable-2m.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (60, 2, 34, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (61, 2, 34, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Cables",
  "item": "/shop/category/cables.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (62, 2, 34, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "USB-C Cable 2 m",
  "item": "/shop/product/cable-2m.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (63, 2, 34, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "USB-C Cable 2 m",
  "sku": "CABLE-2M",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "129",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (64, 2, 35, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Cables",
      "item": "/shop/category/cables.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (65, 2, 35, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (66, 2, 35, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Cables",
  "item": "/shop/category/cables.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (67, 2, 36, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "About us",
      "item": "/shop/info/about.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (68, 2, 36, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (69, 2, 36, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "About us",
  "item": "/shop/info/about.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (70, 2, 37, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Returns",
      "item": "/shop/info/returns.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (71, 2, 37, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (72, 2, 37, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Returns",
  "item": "/shop/info/returns.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (73, 2, 38, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Headphones",
      "item": "/shop/category/headphones.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (74, 2, 38, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (75, 2, 38, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Headphones",
  "item": "/shop/category/headphones.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (76, 2, 39, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Headphones",
      "item": "/shop/category/headphones.html"
    },
    {
      "@type": "ListItem",
      "position": 3,
      "name": "Wireless Headphones ANC",
      "item": "/shop/product/headphones-anc.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (77, 2, 39, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (78, 2, 39, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Headphones",
  "item": "/shop/category/headphones.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (79, 2, 39, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 3,
  "name": "Wireless Headphones ANC",
  "item": "/shop/product/headphones-anc.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (80, 2, 39, 'json-ld', 'Product', '{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "Wireless Headphones ANC",
  "sku": "HEADPHONES-ANC",
  "image": "/shop/img/product.svg",
  "offers": {
    "@type": "Offer",
    "price": "990",
    "priceCurrency": "SEK",
    "availability": "https://schema.org/InStock"
  }
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (81, 2, 40, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Chargers",
      "item": "/shop/category/chargers.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (82, 2, 40, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (83, 2, 40, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Chargers",
  "item": "/shop/category/chargers.html"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (84, 2, 41, 'json-ld', 'BreadcrumbList', '{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {
      "@type": "ListItem",
      "position": 1,
      "name": "Home",
      "item": "/shop/"
    },
    {
      "@type": "ListItem",
      "position": 2,
      "name": "Shipping",
      "item": "/shop/info/shipping.html"
    }
  ]
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (85, 2, 41, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 1,
  "name": "Home",
  "item": "/shop/"
}', NULL);
INSERT INTO structured_data (id, crawl_id, page_url_id, format, type_name, json, errors) VALUES (86, 2, 41, 'json-ld', 'ListItem', '{
  "@type": "ListItem",
  "position": 2,
  "name": "Shipping",
  "item": "/shop/info/shipping.html"
}', NULL);
INSERT INTO texts (id, crawl_id, text) VALUES (1, 1, 'http://127.0.0.1:8000/shop/');
INSERT INTO texts (id, crawl_id, text) VALUES (2, 1, 'http://127.0.0.1:8000/shop/style.css');
INSERT INTO texts (id, crawl_id, text) VALUES (3, 1, 'http://127.0.0.1:8000/shop/variant.css');
INSERT INTO texts (id, crawl_id, text) VALUES (4, 1, 'http://127.0.0.1:8000/shop/img/logo.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (5, 1, 'http://127.0.0.1:8000/shop/img/category.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (6, 1, 'http://127.0.0.1:8000/shop/img/product.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (7, 1, 'http://127.0.0.1:8000/shop/shop.js');
INSERT INTO texts (id, crawl_id, text) VALUES (8, 1, 'http://127.0.0.1:8000/shop/img/favicon.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (9, 1, 'http://127.0.0.1:8000/shop/info/returns.html');
INSERT INTO texts (id, crawl_id, text) VALUES (10, 1, 'http://127.0.0.1:8000/shop/product/cable-2m.html');
INSERT INTO texts (id, crawl_id, text) VALUES (11, 1, 'http://127.0.0.1:8000/shop/info/about.html');
INSERT INTO texts (id, crawl_id, text) VALUES (12, 1, 'http://127.0.0.1:8000/shop/img/banner.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (13, 1, 'http://127.0.0.1:8000/shop/product/charger-20w.html');
INSERT INTO texts (id, crawl_id, text) VALUES (14, 1, 'http://127.0.0.1:8000/shop/product/headphones-anc.html');
INSERT INTO texts (id, crawl_id, text) VALUES (15, 1, 'http://127.0.0.1:8000/shop/product/powerbank-10k.html');
INSERT INTO texts (id, crawl_id, text) VALUES (16, 1, 'http://127.0.0.1:8000/shop/category/chargers.html');
INSERT INTO texts (id, crawl_id, text) VALUES (17, 1, 'http://127.0.0.1:8000/shop/cart');
INSERT INTO texts (id, crawl_id, text) VALUES (18, 1, 'http://127.0.0.1:8000/shop/info/shipping.html');
INSERT INTO texts (id, crawl_id, text) VALUES (19, 1, 'http://127.0.0.1:8000/shop/product/charger-65w.html');
INSERT INTO texts (id, crawl_id, text) VALUES (20, 1, 'http://127.0.0.1:8000/shop/category/cables.html');
INSERT INTO texts (id, crawl_id, text) VALUES (21, 1, 'http://127.0.0.1:8000/shop/category/headphones.html');
INSERT INTO texts (id, crawl_id, text) VALUES (22, 2, 'http://127.0.0.1:8000/shop/');
INSERT INTO texts (id, crawl_id, text) VALUES (23, 2, 'http://127.0.0.1:8000/shop/style.css');
INSERT INTO texts (id, crawl_id, text) VALUES (24, 2, 'http://127.0.0.1:8000/shop/variant.css');
INSERT INTO texts (id, crawl_id, text) VALUES (25, 2, 'http://127.0.0.1:8000/shop/img/category.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (26, 2, 'http://127.0.0.1:8000/shop/img/logo.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (27, 2, 'http://127.0.0.1:8000/shop/img/product.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (28, 2, 'http://127.0.0.1:8000/shop/shop.js');
INSERT INTO texts (id, crawl_id, text) VALUES (29, 2, 'http://127.0.0.1:8000/shop/img/favicon.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (30, 2, 'http://127.0.0.1:8000/shop/product/charger-20w.html');
INSERT INTO texts (id, crawl_id, text) VALUES (31, 2, 'http://127.0.0.1:8000/shop/product/charger-65w.html');
INSERT INTO texts (id, crawl_id, text) VALUES (32, 2, 'http://127.0.0.1:8000/shop/img/banner.svg');
INSERT INTO texts (id, crawl_id, text) VALUES (33, 2, 'http://127.0.0.1:8000/shop/product/powerbank-10k.html');
INSERT INTO texts (id, crawl_id, text) VALUES (34, 2, 'http://127.0.0.1:8000/shop/product/cable-2m.html');
INSERT INTO texts (id, crawl_id, text) VALUES (35, 2, 'http://127.0.0.1:8000/shop/category/cables.html');
INSERT INTO texts (id, crawl_id, text) VALUES (36, 2, 'http://127.0.0.1:8000/shop/info/about.html');
INSERT INTO texts (id, crawl_id, text) VALUES (37, 2, 'http://127.0.0.1:8000/shop/info/returns.html');
INSERT INTO texts (id, crawl_id, text) VALUES (38, 2, 'http://127.0.0.1:8000/shop/category/headphones.html');
INSERT INTO texts (id, crawl_id, text) VALUES (39, 2, 'http://127.0.0.1:8000/shop/product/headphones-anc.html');
INSERT INTO texts (id, crawl_id, text) VALUES (40, 2, 'http://127.0.0.1:8000/shop/category/chargers.html');
INSERT INTO texts (id, crawl_id, text) VALUES (41, 2, 'http://127.0.0.1:8000/shop/info/shipping.html');
INSERT INTO texts (id, crawl_id, text) VALUES (42, 2, 'http://127.0.0.1:8000/shop/cart');
INSERT INTO texts (id, crawl_id, text) VALUES (43, 1, 'Robin Electronics');
INSERT INTO texts (id, crawl_id, text) VALUES (44, 1, 'Cart');
INSERT INTO texts (id, crawl_id, text) VALUES (45, 1, 'Chargers');
INSERT INTO texts (id, crawl_id, text) VALUES (46, 1, 'Cables');
INSERT INTO texts (id, crawl_id, text) VALUES (47, 1, 'Headphones');
INSERT INTO texts (id, crawl_id, text) VALUES (48, 1, 'Shipping');
INSERT INTO texts (id, crawl_id, text) VALUES (49, 1, 'Shop chargers');
INSERT INTO texts (id, crawl_id, text) VALUES (50, 1, 'USB-C Charger 20W 199 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (51, 1, 'GaN Charger 65W 379 kr 449 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (52, 1, 'USB-C Cable 2 m 129 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (53, 1, 'Wireless Headphones ANC 990 kr 1290 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (54, 1, 'Power Bank 10000 mAh 349 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (55, 1, 'Returns');
INSERT INTO texts (id, crawl_id, text) VALUES (56, 1, 'Our story');
INSERT INTO texts (id, crawl_id, text) VALUES (57, 1, 'Home');
INSERT INTO texts (id, crawl_id, text) VALUES (58, 1, 'USB-C Cable 2 m');
INSERT INTO texts (id, crawl_id, text) VALUES (59, 1, 'About us');
INSERT INTO texts (id, crawl_id, text) VALUES (60, 1, 'USB-C Charger 20W');
INSERT INTO texts (id, crawl_id, text) VALUES (61, 1, 'Wireless Headphones ANC');
INSERT INTO texts (id, crawl_id, text) VALUES (62, 1, 'Power Bank 10000 mAh');
INSERT INTO texts (id, crawl_id, text) VALUES (63, 1, '1');
INSERT INTO texts (id, crawl_id, text) VALUES (64, 1, 'GaN Charger 65W');
INSERT INTO texts (id, crawl_id, text) VALUES (65, 2, 'Robin Electronics');
INSERT INTO texts (id, crawl_id, text) VALUES (66, 2, 'Cart');
INSERT INTO texts (id, crawl_id, text) VALUES (67, 2, 'Chargers');
INSERT INTO texts (id, crawl_id, text) VALUES (68, 2, 'Cables');
INSERT INTO texts (id, crawl_id, text) VALUES (69, 2, 'Headphones');
INSERT INTO texts (id, crawl_id, text) VALUES (70, 2, 'Shipping');
INSERT INTO texts (id, crawl_id, text) VALUES (71, 2, 'Shop chargers');
INSERT INTO texts (id, crawl_id, text) VALUES (72, 2, 'USB-C Charger 20W 199 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (73, 2, 'GaN Charger 65W 379 kr 449 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (74, 2, 'USB-C Cable 2 m 129 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (75, 2, 'Wireless Headphones ANC 990 kr 1290 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (76, 2, 'Power Bank 10000 mAh 349 kr');
INSERT INTO texts (id, crawl_id, text) VALUES (77, 2, 'Returns');
INSERT INTO texts (id, crawl_id, text) VALUES (78, 2, 'Our story');
INSERT INTO texts (id, crawl_id, text) VALUES (79, 2, 'Home');
INSERT INTO texts (id, crawl_id, text) VALUES (80, 2, 'USB-C Charger 20W');
INSERT INTO texts (id, crawl_id, text) VALUES (81, 2, 'GaN Charger 65W');
INSERT INTO texts (id, crawl_id, text) VALUES (82, 2, 'Power Bank 10000 mAh');
INSERT INTO texts (id, crawl_id, text) VALUES (83, 2, 'USB-C Cable 2 m');
INSERT INTO texts (id, crawl_id, text) VALUES (84, 2, '1');
INSERT INTO texts (id, crawl_id, text) VALUES (85, 2, 'About us');
INSERT INTO texts (id, crawl_id, text) VALUES (86, 2, 'Wireless Headphones ANC');
INSERT INTO texts (id, crawl_id, text) VALUES (87, 1, '');
INSERT INTO texts (id, crawl_id, text) VALUES (88, 2, '');
INSERT INTO texts (id, crawl_id, text) VALUES (89, 1, 'landmark-unique');
INSERT INTO texts (id, crawl_id, text) VALUES (90, 1, 'heading-order');
INSERT INTO texts (id, crawl_id, text) VALUES (91, 2, 'landmark-unique');
INSERT INTO texts (id, crawl_id, text) VALUES (92, 2, 'heading-order');
INSERT INTO texts (id, crawl_id, text) VALUES (93, 1, 'moderate');
INSERT INTO texts (id, crawl_id, text) VALUES (94, 2, 'moderate');
INSERT INTO texts (id, crawl_id, text) VALUES (95, 1, '.main-nav');
INSERT INTO texts (id, crawl_id, text) VALUES (96, 1, '.footer-column:nth-child(1) > h3');
INSERT INTO texts (id, crawl_id, text) VALUES (97, 2, '.main-nav');
INSERT INTO texts (id, crawl_id, text) VALUES (98, 2, '.footer-column:nth-child(1) > h3');
INSERT INTO texts (id, crawl_id, text) VALUES (99, 1, '<nav class="main-nav">');
INSERT INTO texts (id, crawl_id, text) VALUES (100, 1, '<h3 class="footer-column__title">Customer service</h3>');
INSERT INTO texts (id, crawl_id, text) VALUES (101, 2, '<nav class="main-nav">');
INSERT INTO texts (id, crawl_id, text) VALUES (102, 2, '<h3 class="footer-column__title">Customer service</h3>');
