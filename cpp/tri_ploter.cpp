#include <windows.h>
#include <iostream>
#include <vector>
#include <wingdi.h>
#include <d3d11.h>

using namespace std;

HWND gobal_wallpaper = nullptr;
HDC hdc = nullptr;
vector<MONITORINFO> monitors(0);

BOOL CALLBACK EnumWindowsProc(HWND hwnd, LPARAM lParam)

{
    HWND p = FindWindowEx(hwnd, NULL, "SHELLDLL_DefView", NULL);
    HWND *ret = (HWND *)lParam;

    if (p)
    {
        // Gets the WorkerW Window after the current one.
        *ret = FindWindowEx(NULL, hwnd, "WorkerW", NULL);
    }
    return true;
}

HWND get_wallpaper_window()
{
    // Fetch the Progman window
    HWND progman = FindWindow("ProgMan", NULL);
    // Send 0x052C to Progman. This message directs Progman to spawn a
    // WorkerW behind the desktop icons. If it is already there, nothing
    // happens.
    SendMessageTimeout(progman, 0x052C, 0, 0, SMTO_NORMAL, 1000, nullptr);
    // We enumerate all Windows, until we find one, that has the SHELLDLL_DefView
    // as a child.
    // If we found that window, we take its next sibling and assign it to workerw.
    HWND wallpaper_hwnd = nullptr;
    EnumWindows(EnumWindowsProc, (LPARAM)&wallpaper_hwnd);
    // Return the handle you're looking for.
    return wallpaper_hwnd;
}

BOOL Monitorenumproc(HMONITOR monitor, HDC unnamedParam2, LPRECT unnamedParam3, LPARAM unnamedParam4)
{
    MONITORINFO info;
    info.cbSize = sizeof(MONITORINFO);
    GetMonitorInfo(monitor, &info);

    monitors.push_back(info);

    return true;
}

void analizeMonitors()
{
    EnumDisplayMonitors(nullptr, nullptr, Monitorenumproc, 0);
}

double map(double StartRangeSrc, double EndRangeSrc, double StartRangeDst, double EndRangeDst, double val)
{
    return StartRangeDst + ((EndRangeDst - StartRangeDst) / (EndRangeSrc - StartRangeSrc)) * (val - StartRangeSrc);
}

POINT trsfrm(POINT src, RECT dst, int maxX, int maxY)
{
    long x = long(map(0, maxX, dst.left, dst.right, src.x));
    long y = long(map(0, maxY, dst.top, dst.bottom, src.y));

    // cout << src.x << "," << src.y << ":" << x << "," << y << endl;

    return POINT{x, y};
}

void GetDesktopResolution(int &horizontal, int &vertical)
{
    RECT desktop;
    // Get a handle to the desktop window
    const HWND hDesktop = GetDesktopWindow();
    // Get the size of screen to the variable desktop
    GetWindowRect(hDesktop, &desktop);
    // The top left corner will have coordinates (0,0)
    // and the bottom right corner will have coordinates
    // (horizontal, vertical)
    horizontal = desktop.right;
    vertical = desktop.bottom;
}

bool in_bound(POINT p, RECT r)
{
    return r.top <= p.y && r.bottom >= p.y && r.left <= p.x && r.right >= p.x;
}

void draw_tri(int x1, int y1, int x2, int y2, int x3, int y3, int maxX, int maxY, int r, int g, int b)
{

    if (gobal_wallpaper == nullptr)
    {
        gobal_wallpaper = get_wallpaper_window();
        hdc = GetDC(gobal_wallpaper);
    }
    if (monitors.size() == 0)
    {
        analizeMonitors();
        double q = 1.25;
        for (int i = 0; i < monitors.size(); i++)
        {
            monitors[i].rcMonitor.right *= q;
            monitors[i].rcMonitor.left *= q;
            monitors[i].rcMonitor.top *= q;
            monitors[i].rcMonitor.bottom *= q;
        }
    }
#define OLD_TRI
#ifdef OLD_TRI
    SetDCBrushColor(hdc, RGB(r, g, b));
    SelectObject(hdc, GetStockObject(DC_BRUSH));
    SetDCPenColor(hdc, RGB(r, g, b));
    SelectObject(hdc, GetStockObject(DC_PEN));

    for (auto m : monitors)
    {

        POINT p1 = {x1, y1};
        POINT p2 = {x2, y2};
        POINT p3 = {x3, y3};
        POINT ps[3] = {
            trsfrm(p1, m.rcMonitor, maxX, maxY),
            trsfrm(p2, m.rcMonitor, maxX, maxY),
            trsfrm(p3, m.rcMonitor, maxX, maxY),

        };

        Polygon(hdc, ps, 3);
    }
#else
    for (auto m : monitors)
    {

        POINT p1 = {x1, y1};
        POINT p2 = {x2, y2};
        POINT p3 = {x3, y3};

        POINT ps[3] = {
            trsfrm(p1, m.rcMonitor, maxX, maxY),
            trsfrm(p2, m.rcMonitor, maxX, maxY),
            trsfrm(p3, m.rcMonitor, maxX, maxY),
        };

        if (PtVisible(hdc, ps[0].x, ps[0].y) || PtVisible(hdc, ps[1].x, ps[1].y) || PtVisible(hdc, ps[2].x, ps[2].y))
        {

            TRIVERTEX vertex[3];
            vertex[0].x = ps[0].x;
            vertex[0].y = ps[0].y;
            vertex[0].Red = r << 8;
            vertex[0].Green = g << 8;
            vertex[0].Blue = b << 8;
            vertex[0].Alpha = 0x0000;

            vertex[1].x = ps[1].x;
            vertex[1].y = ps[1].y;
            vertex[1].Red = r << 8;
            vertex[1].Green = g << 8;
            vertex[1].Blue = b << 8;
            vertex[1].Alpha = 0x0000;

            vertex[2].x = ps[2].x;
            vertex[2].y = ps[2].y;
            vertex[2].Red = r << 8;
            vertex[2].Green = g << 8;
            vertex[2].Blue = b << 8;
            vertex[2].Alpha = 0x0000;

            GRADIENT_TRIANGLE gTriangle;
            gTriangle.Vertex1 = 0;
            gTriangle.Vertex2 = 1;
            gTriangle.Vertex3 = 2;

            GradientFill(hdc, vertex, 3, &gTriangle, 1, GRADIENT_FILL_TRIANGLE);
        }
    }

#endif
}
