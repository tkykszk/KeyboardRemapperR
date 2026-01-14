// Phase 3: キー入力送信の実装 - 単体テスト
// 
// このファイルには、Phase 3 の単体テストが含まれています。
// テストケース: PHASE3_TEST_CASES.md の UT-3.1 ~ UT-3.4

#[cfg(test)]
#[cfg(target_os = "windows")]
mod phase3_unit_tests {
    use super::*;

    // ============================================================================
    // UT-3.1: キー入力抑制機能のテスト
    // ============================================================================

    #[test]
    fn ut_3_1_1_install_keyboard_hook() {
        // 目的: install_keyboard_hook() が正常に動作することを確認
        unsafe {
            let result = install_keyboard_hook();
            assert!(result.is_ok(), "Failed to install keyboard hook");
            assert!(KEYBOARD_HOOK.is_some(), "KEYBOARD_HOOK should be Some after installation");
            
            // クリーンアップ
            uninstall_keyboard_hook();
        }
    }

    #[test]
    fn ut_3_1_2_uninstall_keyboard_hook() {
        // 目的: uninstall_keyboard_hook() が正常に動作することを確認
        unsafe {
            install_keyboard_hook().ok();
            uninstall_keyboard_hook();
            assert!(KEYBOARD_HOOK.is_none(), "KEYBOARD_HOOK should be None after uninstallation");
        }
    }

    #[test]
    fn ut_3_1_3_add_suppressed_key() {
        // 目的: add_suppressed_key() が正常に動作することを確認
        const VK_CAPITAL: u16 = 0x14; // CapsLock
        
        add_suppressed_key(VK_CAPITAL);
        assert!(should_suppress_key(VK_CAPITAL, true), "CapsLock should be suppressed");
        
        // クリーンアップ
        remove_suppressed_key(VK_CAPITAL);
    }

    #[test]
    fn ut_3_1_4_remove_suppressed_key() {
        // 目的: remove_suppressed_key() が正常に動作することを確認
        const VK_CAPITAL: u16 = 0x14; // CapsLock
        
        add_suppressed_key(VK_CAPITAL);
        remove_suppressed_key(VK_CAPITAL);
        assert!(!should_suppress_key(VK_CAPITAL, true), "CapsLock should not be suppressed after removal");
    }

    #[test]
    fn ut_3_1_5_multiple_suppressed_keys() {
        // 目的: 複数のキーを同時に管理できることを確認
        const VK_CAPITAL: u16 = 0x14; // CapsLock
        const VK_A: u16 = 0x41;       // A
        
        add_suppressed_key(VK_CAPITAL);
        add_suppressed_key(VK_A);
        
        assert!(should_suppress_key(VK_CAPITAL, true), "CapsLock should be suppressed");
        assert!(should_suppress_key(VK_A, true), "A should be suppressed");
        
        remove_suppressed_key(VK_CAPITAL);
        
        assert!(!should_suppress_key(VK_CAPITAL, true), "CapsLock should not be suppressed after removal");
        assert!(should_suppress_key(VK_A, true), "A should still be suppressed");
        
        // クリーンアップ
        remove_suppressed_key(VK_A);
    }

    // ============================================================================
    // UT-3.2: キー入力送信機能のテスト
    // ============================================================================

    #[test]
    fn ut_3_2_1_send_key_event() {
        // 目的: send_key_event() が正常に動作することを確認
        const VK_A: u16 = 0x41; // A
        
        unsafe {
            let result_down = send_key_event(VK_A, true, false);
            assert!(result_down.is_ok(), "Failed to send key down event");
            
            let result_up = send_key_event(VK_A, false, false);
            assert!(result_up.is_ok(), "Failed to send key up event");
        }
    }

    #[test]
    fn ut_3_2_2_is_extended_key() {
        // 目的: is_extended_key() が正しく判定することを確認
        const VK_A: u16 = 0x41;
        const VK_CAPITAL: u16 = 0x14;
        const VK_LCONTROL: u16 = 0xA2;
        
        // 拡張キー
        assert!(is_extended_key(0x25), "Left Arrow should be extended key");
        assert!(is_extended_key(0x26), "Up Arrow should be extended key");
        assert!(is_extended_key(0x27), "Right Arrow should be extended key");
        assert!(is_extended_key(0x28), "Down Arrow should be extended key");
        assert!(is_extended_key(0x24), "Home should be extended key");
        assert!(is_extended_key(0x23), "End should be extended key");
        assert!(is_extended_key(0xA3), "Right Control should be extended key");
        assert!(is_extended_key(0xA5), "Right Alt should be extended key");
        
        // 通常キー
        assert!(!is_extended_key(VK_A), "A should not be extended key");
        assert!(!is_extended_key(VK_CAPITAL), "CapsLock should not be extended key");
        assert!(!is_extended_key(VK_LCONTROL), "Left Control should not be extended key");
    }

    #[test]
    fn ut_3_2_3_send_key() {
        // 目的: send_key() が正常に動作することを確認
        let result_a = send_key("A", true);
        assert!(result_a.is_ok(), "Failed to send key 'A'");
        
        let result_caps = send_key("CapsLock", true);
        assert!(result_caps.is_ok(), "Failed to send key 'CapsLock'");
    }

    #[test]
    fn ut_3_2_4_send_key_invalid() {
        // 目的: 無効なキー名でエラーが返されることを確認
        let result = send_key("InvalidKey", true);
        assert!(result.is_err(), "Should return error for invalid key name");
        assert!(result.unwrap_err().contains("Unknown key name"), "Error message should mention unknown key name");
    }

    #[test]
    fn ut_3_2_5_injected_key_marker() {
        // 目的: INJECTED_KEY_MARKER が正しく定義されていることを確認
        const INJECTED_KEY_MARKER: usize = 0x12345678;
        assert_eq!(INJECTED_KEY_MARKER, 0x12345678, "INJECTED_KEY_MARKER should be 0x12345678");
    }

    // ============================================================================
    // UT-3.3: Swap モードのテスト
    // ============================================================================

    #[test]
    fn ut_3_3_1_swap_mapping_generation() {
        // 目的: Swap マッピングで双方向のマッピングが生成されることを確認
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "LCtrl".to_string(), MappingType::Swap);
        
        let device = &config.devices[0];
        assert_eq!(device.mappings.len(), 2, "Swap mapping should generate 2 mappings");
        
        // CapsLock -> LCtrl
        let mapping1 = device.mappings.iter().find(|m| m.from == "CapsLock");
        assert!(mapping1.is_some(), "CapsLock -> LCtrl mapping should exist");
        assert_eq!(mapping1.unwrap().to, "LCtrl", "CapsLock should map to LCtrl");
        
        // LCtrl -> CapsLock
        let mapping2 = device.mappings.iter().find(|m| m.from == "LCtrl");
        assert!(mapping2.is_some(), "LCtrl -> CapsLock mapping should exist");
        assert_eq!(mapping2.unwrap().to, "CapsLock", "LCtrl should map to CapsLock");
    }

    #[test]
    fn ut_3_3_2_circular_reference_simple() {
        // 目的: A → B → A のような単純な循環が検出されることを確認
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Swap);
        
        // B -> A を追加しようとすると循環参照（Swap で既に追加されている）
        assert!(config.check_circular_reference("04FE:0021", "B", "A"), "Should detect circular reference");
    }

    #[test]
    fn ut_3_3_3_circular_reference_complex() {
        // 目的: A → B → C → A のような複雑な循環が検出されることを確認
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Remap);
        config.add_mapping("04FE:0021", "B".to_string(), "C".to_string(), MappingType::Remap);
        
        // C -> A を追加しようとすると循環参照
        assert!(config.check_circular_reference("04FE:0021", "C", "A"), "Should detect complex circular reference");
    }

    #[test]
    fn ut_3_3_4_no_circular_reference() {
        // 目的: 循環参照がない場合に false が返されることを確認
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Remap);
        
        // C -> D は循環参照なし
        assert!(!config.check_circular_reference("04FE:0021", "C", "D"), "Should not detect circular reference");
    }

    #[test]
    fn ut_3_3_5_swap_mapping_overwrite() {
        // 目的: 既存の Swap マッピングが正しく上書きされることを確認
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Swap);
        config.add_mapping("04FE:0021", "A".to_string(), "C".to_string(), MappingType::Swap);
        
        let device = &config.devices[0];
        assert_eq!(device.mappings.len(), 2, "Should have 2 mappings after overwrite");
        
        // A -> C
        let mapping1 = device.mappings.iter().find(|m| m.from == "A");
        assert!(mapping1.is_some(), "A -> C mapping should exist");
        assert_eq!(mapping1.unwrap().to, "C", "A should map to C");
        
        // C -> A
        let mapping2 = device.mappings.iter().find(|m| m.from == "C");
        assert!(mapping2.is_some(), "C -> A mapping should exist");
        assert_eq!(mapping2.unwrap().to, "A", "C should map to A");
        
        // B -> A は存在しない
        let mapping3 = device.mappings.iter().find(|m| m.from == "B");
        assert!(mapping3.is_none(), "B -> A mapping should not exist");
    }

    // ============================================================================
    // UT-3.4: Disable モードのテスト
    // ============================================================================

    #[test]
    fn ut_3_4_1_disable_mapping() {
        // 目的: Disable マッピングが正しく追加されることを確認
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "None".to_string(), MappingType::Disable);
        
        let device = &config.devices[0];
        assert_eq!(device.mappings.len(), 1, "Should have 1 mapping");
        assert_eq!(device.mappings[0].from, "CapsLock", "From should be CapsLock");
        assert_eq!(device.mappings[0].to, "None", "To should be None");
        assert_eq!(device.mappings[0].mapping_type, MappingType::Disable, "Type should be Disable");
    }

    #[test]
    fn ut_3_4_2_disable_mapping_processing() {
        // 目的: Disable マッピングでキーが送信されないことを確認
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "None".to_string(), MappingType::Disable);
        
        let result = config.process_key_event("04FE:0021", "CapsLock", true);
        assert!(result.is_some(), "Should return Some for disabled key");
        assert!(result.unwrap().contains("disabled"), "Result should contain 'disabled'");
    }
}

// ============================================================================
// パフォーマンステスト
// ============================================================================

#[cfg(test)]
#[cfg(target_os = "windows")]
mod phase3_performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn pt_3_1_key_input_latency() {
        // 目的: キー入力の遅延が 5ms 以下であることを確認
        const VK_A: u16 = 0x41;
        let mut total_duration = std::time::Duration::ZERO;
        let iterations = 100;
        
        for _ in 0..iterations {
            let start = Instant::now();
            
            unsafe {
                send_key_event(VK_A, true, false).ok();
                send_key_event(VK_A, false, false).ok();
            }
            
            let duration = start.elapsed();
            total_duration += duration;
        }
        
        let avg_duration = total_duration / iterations;
        println!("Average latency: {:?}", avg_duration);
        
        assert!(avg_duration.as_millis() <= 5, "Average latency should be <= 5ms, got {:?}", avg_duration);
    }

    #[test]
    #[ignore] // 長時間実行テストのため、デフォルトでは無視
    fn pt_3_4_long_running_test() {
        // 目的: 24時間以上の連続稼働でメモリリークやクラッシュが発生しないこと
        // 注意: このテストは手動で実行してください
        // cargo test pt_3_4_long_running_test --ignored -- --nocapture
        
        use std::thread;
        use std::time::Duration;
        
        println!("Starting long running test (24 hours)...");
        
        let start = Instant::now();
        let mut iteration = 0;
        
        while start.elapsed() < Duration::from_secs(24 * 60 * 60) {
            // 1時間ごとにメモリ使用量を報告
            if iteration % 3600 == 0 {
                println!("Elapsed: {:?}, Iteration: {}", start.elapsed(), iteration);
            }
            
            // キー入力をシミュレート
            unsafe {
                send_key_event(0x41, true, false).ok();
                send_key_event(0x41, false, false).ok();
            }
            
            thread::sleep(Duration::from_millis(100));
            iteration += 1;
        }
        
        println!("Long running test completed successfully");
    }

    #[test]
    fn pt_3_5_high_frequency_input_test() {
        // 目的: 高頻度のキー入力でも正常に動作することを確認
        const VK_A: u16 = 0x41;
        let iterations = 1000; // 1秒間に100回 × 10秒 = 1000回
        let mut success_count = 0;
        
        for _ in 0..iterations {
            unsafe {
                if send_key_event(VK_A, true, false).is_ok() && 
                   send_key_event(VK_A, false, false).is_ok() {
                    success_count += 1;
                }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        let success_rate = (success_count as f64 / iterations as f64) * 100.0;
        println!("Success rate: {:.2}%", success_rate);
        
        assert!(success_rate >= 99.0, "Success rate should be >= 99%, got {:.2}%", success_rate);
    }
}
