use std::process::Command;

pub struct Core;

impl Core {
    pub fn eject() -> bool {
        println!("Початок витягування всіх зовнішніх накопичувачів.");

        #[cfg(target_os = "windows")]
        return Self::eject_windows();

        #[cfg(target_os = "macos")]
        return Self::eject_macos();

        #[cfg(target_os = "linux")]
        return Self::eject_linux();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return false;
    }

    fn eject_via_shell(drive: &str) -> bool {
        let result = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(r#"
                try {{
                    Write-Host "Спроба витягування {} через Shell.Application..." -ForegroundColor Yellow

                    $shell = New-Object -ComObject Shell.Application
                    $folder = $shell.Namespace(17)
                    $item = $folder.ParseName("{}")

                    if($item -ne $null) {{
                        $verbs = $item.Verbs()
                        Write-Host "Знайдено $($verbs.Count) доступних дій для диска"

                        foreach($verb in $verbs) {{
                            Write-Host "Доступна дія: $($verb.Name)"
                        }}

                        $ejectVerb = $verbs | Where-Object {{
                            $_.Name -like "*ject*" -or
                            $_.Name -like "*Безпечне видалення*" -or
                            $_.Name -like "*Eject*" -or
                            $_.Name -like "*Извлечь*" -or
                            $_.Name -like "*Safely Remove*" -or
                            $_.Name -like "*відключ*" -or
                            $_.Name -like "*видал*"
                        }}

                        if($ejectVerb) {{
                            Write-Host "Використовуємо дію: $($ejectVerb.Name)" -ForegroundColor Green
                            $ejectVerb.DoIt()

                            # Чекаємо та перевіряємо результат
                            Start-Sleep -Seconds 3

                            $checkDrive = Get-WmiObject -Class Win32_LogicalDisk | Where-Object {{$_.DeviceID -eq "{}"}}
                            if(-not $checkDrive) {{
                                Write-Host "SHELL_SUCCESS_VERIFIED"
                            }} else {{
                                Write-Host "SHELL_ACTION_FAILED"
                            }}
                        }} else {{
                            Write-Host "Не знайдено дії витягування для диска {}"
                            Write-Host "SHELL_NO_EJECT_VERB"
                        }}
                    }} else {{
                        Write-Host "Диск {} не знайдено в Shell"
                        Write-Host "SHELL_ITEM_NOT_FOUND"
                    }}
                }} catch {{
                    Write-Host "SHELL_ERROR: $_" -ForegroundColor Red
                }}
                "#, drive, drive, drive, drive, drive)
            ])
            .output();

        match result {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("Shell output: {}", output_str); // Додаткове логування
                output_str.contains("SHELL_SUCCESS_VERIFIED")
            }
            Err(e) => {
                println!("Shell command error: {}", e);
                false
            }
        }
    }

    fn eject_via_diskpart(drive_letter: &str) -> bool {
        let drive_letter = drive_letter.replace(":", "");

        let result = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(r#"
                try {{
                    # Створюємо тимчасовий файл для diskpart команд
                    $tempFile = [System.IO.Path]::GetTempFileName()
                    "select volume {}" | Out-File -FilePath $tempFile -Encoding ASCII
                    "remove" | Out-File -FilePath $tempFile -Encoding ASCII -Append
                    "exit" | Out-File -FilePath $tempFile -Encoding ASCII -Append

                    # Запускаємо diskpart
                    $process = Start-Process -FilePath "diskpart.exe" -ArgumentList "/s `"$tempFile`"" -Wait -PassThru -WindowStyle Hidden

                    # Видаляємо тимчасовий файл
                    Remove-Item $tempFile -ErrorAction SilentlyContinue

                    if($process.ExitCode -eq 0) {{
                        Write-Host "DISKPART_SUCCESS"
                    }} else {{
                        Write-Host "DISKPART_FAILED"
                    }}
                }} catch {{
                    Write-Host "DISKPART_ERROR: $_"
                }}
                "#, drive_letter)
            ])
            .output();

        match result {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains("DISKPART_SUCCESS")
            }
            Err(_) => false
        }
    }

    fn eject_via_dismount(drive_letter: &str) -> bool {
        let result = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(r#"
                try {{
                    Dismount-Volume -DriveLetter "{}" -Force
                    Write-Host "DISMOUNT_SUCCESS"
                }} catch {{
                    Write-Host "DISMOUNT_ERROR: $_"
                }}
                "#, drive_letter.replace(":", ""))
            ])
            .output();

        match result {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains("DISMOUNT_SUCCESS")
            }
            Err(_) => false
        }
    }

    fn eject_windows() -> bool {
        println!("Windows: пошук всіх зовнішніх дисків...");

        let output = Command::new("powershell")
            .args(&[
                "-Command",
                r#"
            # Отримуємо тільки справжні зовнішні накопичувачі
            $removableDisks = @()

            # Метод 1: Через Win32_LogicalDisk (DriveType 2 = знімний)
            $logicalDisks = Get-WmiObject -Class Win32_LogicalDisk | Where-Object {$_.DriveType -eq 2}

            # Метод 2: Перевіряємо фізичні диски на предмет зовнішнього підключення
            foreach($disk in $logicalDisks) {
                try {
                    # Знаходимо відповідний фізичний диск
                    $partition = Get-WmiObject -Query "ASSOCIATORS OF {Win32_LogicalDisk.DeviceID='$($disk.DeviceID)'} WHERE AssocClass=Win32_LogicalDiskToPartition"
                    if($partition) {
                        $physicalDisk = Get-WmiObject -Query "ASSOCIATORS OF {Win32_DiskPartition.DeviceID='$($partition.DeviceID)'} WHERE AssocClass=Win32_DiskDriveToDiskPartition"

                        # Перевіряємо, чи це справді зовнішній пристрій
                        if($physicalDisk -and (
                            $physicalDisk.InterfaceType -eq "USB" -or
                            $physicalDisk.MediaType -like "*Removable*" -or
                            $physicalDisk.MediaType -like "*External*" -or
                            $physicalDisk.PNPDeviceID -like "*USB*" -or
                            $physicalDisk.PNPDeviceID -like "*USBSTOR*"
                        )) {
                            $removableDisks += [PSCustomObject]@{
                                DriveLetter = $disk.DeviceID
                                Label = $disk.VolumeName
                                Size = [math]::Round($disk.Size / 1GB, 2)
                                InterfaceType = $physicalDisk.InterfaceType
                                Model = $physicalDisk.Model
                                PNPDeviceID = $physicalDisk.PNPDeviceID
                            }
                        }
                    }
                } catch {
                    # Якщо не вдалося отримати детальну інформацію, але DriveType = 2, то це знімний диск
                    $removableDisks += [PSCustomObject]@{
                        DriveLetter = $disk.DeviceID
                        Label = $disk.VolumeName
                        Size = [math]::Round($disk.Size / 1GB, 2)
                        InterfaceType = "Unknown"
                        Model = "Unknown"
                        PNPDeviceID = "Unknown"
                    }
                }
            }

            if($removableDisks.Count -eq 0) {
                Write-Host "NO_EXTERNAL_DRIVES"
            } else {
                Write-Host "FOUND_DRIVES:$($removableDisks.Count)"
                foreach($drive in $removableDisks) {
                    Write-Host "DRIVE:$($drive.DriveLetter)|$($drive.Label)|$($drive.Size)GB|$($drive.InterfaceType)|$($drive.Model)"
                }
            }
            "#
            ])
            .output();

        let drive_list: Vec<String> = match output {
            Ok(output) => String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|s| {
                    let s = s.trim();
                    if s.starts_with("DRIVE:") {
                        s.split("|").next().map(|part| part[6..].to_string())
                    }
                    else {
                        None
                    }
                })
                .collect(),
            Err(e) => {
                println!("Помилка отримання списку дисків: {}", e);
                return false;
            }
        };

        if drive_list.is_empty() {
            print!("Зовнішніх дисків не знайдено.");
            return true;
        }

        let mut success_count = 0;
        for drive in drive_list.iter() {
            if Self::eject_via_shell(drive)
                || Self::eject_via_diskpart(drive)
                || Self::eject_via_dismount(drive)
            {
                println!("Накопичувач {} успішно вийнято!", drive);
                success_count += 1;
            }
            else {
                println!("Не вдалося вийняти накопичувач {}.", drive);
            }
        }

        println!("Результат {}/{}", success_count, drive_list.len());
        success_count > 0 || drive_list.is_empty()
    }

    fn eject_macos() -> bool {
        false
    }

    fn eject_linux() -> bool {
        false
    }
}