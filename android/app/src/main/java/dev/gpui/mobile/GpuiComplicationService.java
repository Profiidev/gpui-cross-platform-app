package dev.gpui.mobile;

import android.app.PendingIntent;
import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.content.pm.PackageManager;
import android.os.RemoteException;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.wear.watchface.complications.data.ComplicationData;
import androidx.wear.watchface.complications.data.ComplicationText;
import androidx.wear.watchface.complications.data.ComplicationType;
import androidx.wear.watchface.complications.data.LongTextComplicationData;
import androidx.wear.watchface.complications.data.PlainComplicationText;
import androidx.wear.watchface.complications.data.RangedValueComplicationData;
import androidx.wear.watchface.complications.data.ShortTextComplicationData;
import androidx.wear.watchface.complications.datasource.ComplicationDataSourceService;
import androidx.wear.watchface.complications.datasource.ComplicationDataSourceUpdateRequester;
import androidx.wear.watchface.complications.datasource.ComplicationRequest;

/**
 * Wear OS Watch Face Complication data source backed by Rust JNI callbacks.
 * Supports SHORT_TEXT, LONG_TEXT, and RANGED_VALUE watch face slots.
 */
public class GpuiComplicationService extends ComplicationDataSourceService {
    private static volatile boolean sLibLoaded = false;

    static synchronized void ensureNativeLoaded(Context context) {
        if (sLibLoaded) return;
        try {
            ComponentName cn = new ComponentName(context, GpuiActivity.class);
            ActivityInfo ai = context.getPackageManager().getActivityInfo(
                    cn, PackageManager.GET_META_DATA);
            String lib = ai.metaData != null ? ai.metaData.getString("android.app.lib_name") : null;
            if (lib != null) {
                System.loadLibrary(lib);
                sLibLoaded = true;
            }
        } catch (PackageManager.NameNotFoundException | UnsatisfiedLinkError ignored) {
            sLibLoaded = true;
        }
    }

    /** Request an immediate refresh of all active watch face complications for this app. */
    public static void requestUpdate(Context context) {
        ComponentName provider = new ComponentName(context, GpuiComplicationService.class);
        ComplicationDataSourceUpdateRequester.create(context, provider).requestUpdateAll();
    }

    @Nullable
    @Override
    public ComplicationData getPreviewData(@NonNull ComplicationType type) {
        ensureNativeLoaded(this);
        return buildComplicationData(
                type,
                safeGetTitle(),
                safeGetValue(),
                safeGetSubtitle(),
                null);
    }

    @Override
    public void onComplicationRequest(
            @NonNull ComplicationRequest request,
            @NonNull ComplicationRequestListener listener) {
        ensureNativeLoaded(this);
        Intent launchIntent = new Intent(this, GpuiActivity.class);
        PendingIntent tapAction = PendingIntent.getActivity(
                this,
                0,
                launchIntent,
                PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE);

        ComplicationData data = buildComplicationData(
                request.getComplicationType(),
                safeGetTitle(),
                safeGetValue(),
                safeGetSubtitle(),
                tapAction);
        try {
            listener.onComplicationData(data);
        } catch (RemoteException ignored) {
        }
    }

    @Nullable
    private ComplicationData buildComplicationData(
            @NonNull ComplicationType type,
            String title,
            String value,
            String subtitle,
            @Nullable PendingIntent tapAction) {
        ComplicationText valueText = new PlainComplicationText.Builder(value).build();
        ComplicationText titleText = new PlainComplicationText.Builder(title).build();
        ComplicationText descText = new PlainComplicationText.Builder(title + ": " + value).build();

        if (type == ComplicationType.SHORT_TEXT) {
            return new ShortTextComplicationData.Builder(valueText, descText)
                    .setTitle(titleText)
                    .setTapAction(tapAction)
                    .build();
        } else if (type == ComplicationType.LONG_TEXT) {
            ComplicationText longText = new PlainComplicationText.Builder(
                    value + " — " + subtitle).build();
            return new LongTextComplicationData.Builder(longText, descText)
                    .setTitle(titleText)
                    .setTapAction(tapAction)
                    .build();
        } else if (type == ComplicationType.RANGED_VALUE) {
            return new RangedValueComplicationData.Builder(100f, 0f, 100f, descText)
                    .setText(valueText)
                    .setTitle(titleText)
                    .setTapAction(tapAction)
                    .build();
        }
        return null;
    }

    static String safeGetTitle() {
        try {
            return nativeGetWidgetTitle();
        } catch (UnsatisfiedLinkError e) {
            return "GPUI";
        }
    }

    static String safeGetValue() {
        try {
            return nativeGetWidgetValue();
        } catch (UnsatisfiedLinkError e) {
            return "Ready";
        }
    }

    static String safeGetSubtitle() {
        try {
            return nativeGetWidgetSubtitle();
        } catch (UnsatisfiedLinkError e) {
            return "Tap to open";
        }
    }

    private static native String nativeGetWidgetTitle();
    private static native String nativeGetWidgetValue();
    private static native String nativeGetWidgetSubtitle();
}
