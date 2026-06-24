/*This file is part of Output Blaster.

Output Blaster is free software : you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Output Blaster is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Output Blaster.If not, see < https://www.gnu.org/licenses/>.*/

#include "Game.h"
#include "../Output Files/BroadcastOutputs.h"
#include <windows.h>
#include <iostream>
#include <fstream>
#include <string>

typedef unsigned char U8;

Helpers * helpers = new Helpers();

int configOutputsSystem = GetPrivateProfileInt(TEXT("Settings"), TEXT("OutputsSystem"), 0, settingsFilename);
int configNetOutputsWithLF = GetPrivateProfileInt(TEXT("Settings"), TEXT("NetOutputsWithLF"), 0, settingsFilename);
int configNetOutputsTCPPort = GetPrivateProfileInt(TEXT("Settings"), TEXT("NetOutputsTCPPort"), 37520, settingsFilename);
int configNetOutputsUDPBroadcastPort = GetPrivateProfileInt(TEXT("Settings"), TEXT("NetOutputsUDPBroadcastPort"), 37521, settingsFilename);

bool Helpers::fileExists(char *filename)
{
	std::ifstream ifile(filename);
	return !ifile.fail();
}

void Helpers::log(char *msg) {
	if (enableLogging == 0) { return; }
	std::ofstream ofs("FFBlog.txt", std::ofstream::app);
	ofs << msg << std::endl;
	ofs.close();
}

void Helpers::logInt(int value) {
	std::string njs = std::to_string(value);
	log((char *)njs.c_str());
}

void Helpers::logInit(char *msg) {
	if (enableLogging == 0) { return; }
	std::ofstream ofs("FFBlog.txt", std::ofstream::out);
	ofs << msg << std::endl;
	ofs.close();
}

// reading memory
LPVOID Helpers::GetTranslatedOffset(INT_PTR offset)
{
	return reinterpret_cast<LPVOID>((INT_PTR)GetModuleHandle(NULL) + offset);
}

UINT8 Helpers::ReadByte(INT_PTR offset, bool isRelativeOffset)
{
	UINT8 val = 0;
	SIZE_T read;
	LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
	ReadProcessMemory(GetCurrentProcess(), trueOffset, &val, sizeof(UINT8), &read);
	return val;
}

float Helpers::WriteFloat32(INT_PTR offset, float val, bool isRelativeOffset)
{
	//val = 0.0f;
	SIZE_T written;
	LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
	WriteProcessMemory(GetCurrentProcess(), trueOffset, &val, sizeof(float), &written);
	return val;
}

UINT8 Helpers::WriteByte(INT_PTR offset, UINT8 val, bool isRelativeOffset)
{
	SIZE_T written;
	LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
	WriteProcessMemory(GetCurrentProcess(), trueOffset, &val, sizeof(UINT8), &written);
	return val;
}

INT_PTR Helpers::WriteIntPtr(INT_PTR offset, INT_PTR val, bool isRelativeOffset)
{
	SIZE_T written;
	LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
	WriteProcessMemory(GetCurrentProcess(), trueOffset, &val, sizeof(INT_PTR), &written);
	return val;
}

UINT8 Helpers::WriteNop(INT_PTR offset, bool isRelativeOffset)
{
	U8 nop = 0x90;
	SIZE_T written;
	LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
	WriteProcessMemory(GetCurrentProcess(), trueOffset, &nop, 1, &written);
	return nop;
}

int Helpers::ReadInt32(INT_PTR offset, bool isRelativeOffset)
{
	int val = 0;
	SIZE_T read;
	LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
	ReadProcessMemory(GetCurrentProcess(), trueOffset, &val, sizeof(int), &read);
	return val;
}

INT_PTR Helpers::ReadIntPtr(INT_PTR offset, bool isRelativeOffset)
{
	SIZE_T read;
	LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
	INT_PTR val;
	ReadProcessMemory(GetCurrentProcess(), trueOffset, &val, sizeof(INT_PTR), &read);
	return val;
}

float Helpers::ReadFloat32(INT_PTR offset, bool isRelativeOffset)
{
		
		float val = 0.0f;
		SIZE_T read;
		LPVOID trueOffset = (isRelativeOffset ? GetTranslatedOffset(offset) : (LPVOID)offset);
		ReadProcessMemory(GetCurrentProcess(), trueOffset, &val, sizeof(float), &read);
		return val;		
}

void Game::OutputsGameLoop()
{
	return;
}

COutputs* Game::CreateOutputsFromConfig()
{
    // Always create WinOutputs (for OutputHooker compat)
    auto winOut = new CWinOutputs();

    switch (configOutputsSystem) {
        case 1: {
            auto netOut = new CNetOutputs();
            netOut->TcpPort = configNetOutputsTCPPort;
            netOut->UdpBroadcastPort = configNetOutputsUDPBroadcastPort;
            if (configNetOutputsWithLF==1) {
                netOut->FrameEnding = std::string("\r\n");
            }
            return new CBroadcastOutputs(winOut, netOut);
        } break;
        case 0:
        default:
            return winOut;
            break;
    }
}

void AutoLaunchWinGame()
{
    int autoLaunch = GetPrivateProfileIntA("Settings", "AutoLaunchWinGame", 1, ".\\OutputBlaster.ini");
    if (!autoLaunch)
    {
        OutputDebugStringA("OB: AutoLaunchWinGame=0 in INI, skipping");
        return;
    }

    if (FindWindowA(NULL, "Arcade Output Display"))
    {
        OutputDebugStringA("OB: WinGame window found, already running");
        return;
    }

    char exeDir[MAX_PATH];
    GetModuleFileNameA(NULL, exeDir, MAX_PATH);
    char* slash = strrchr(exeDir, '\\');
    if (slash) *slash = 0;

    const char* candidates[] = {
        "win-game.exe",
        "..\\win-game.exe",
        "..\\..\\win-game.exe",
        "..\\win-game\\src-tauri\\target\\release\\win-game.exe",
        "..\\..\\win-game\\src-tauri\\target\\release\\win-game.exe",
    };

    for (const char* rel : candidates)
    {
        char full[MAX_PATH];
        sprintf_s(full, "%s\\%s", exeDir, rel);
        char absPath[MAX_PATH];
        GetFullPathNameA(full, MAX_PATH, absPath, NULL);
        if (GetFileAttributesA(absPath) != INVALID_FILE_ATTRIBUTES)
        {
            STARTUPINFOA si = { sizeof(si) };
            PROCESS_INFORMATION pi;
            if (CreateProcessA(absPath, NULL, NULL, NULL, FALSE, 0, NULL, NULL, &si, &pi))
            {
                CloseHandle(pi.hProcess);
                CloseHandle(pi.hThread);
                char buf[256];
                sprintf_s(buf, "OB: Launched win-game.exe from %s", absPath);
                OutputDebugStringA(buf);
            }
            else
            {
                char buf[256];
                sprintf_s(buf, "OB: CreateProcess failed for %s error=%u", absPath, GetLastError());
                OutputDebugStringA(buf);
            }
            return;
        }
    }

    OutputDebugStringA("OB: win-game.exe not found in any candidate path");
}
