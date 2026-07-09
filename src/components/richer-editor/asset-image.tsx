import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import Image from "@tiptap/extension-image";

export type ResolveAssetSrc = (src: string) => string;
export type UploadImage = (file: File) => Promise<string>;

export function createAssetImage(resolveSrc: ResolveAssetSrc) {
  return Image.extend({
    addAttributes() {
      return {
        ...this.parent?.(),
        src: {
          default: null,
          parseHTML: (element) =>
            element.getAttribute("data-canonical-src") ??
            element.getAttribute("src"),
          renderHTML: (attributes) =>
            attributes.src
              ? {
                  src: resolveSrc(attributes.src as string),
                  "data-canonical-src": attributes.src,
                }
              : {},
        },
      };
    },
  }).configure({
    resize: {
      enabled: true,
      minWidth: 80,
      alwaysPreserveAspectRatio: true,
      directions: ["left", "right", "bottom-left", "bottom-right"],
    },
  });
}

export function createImageDropPaste(upload: UploadImage | undefined) {
  return Extension.create({
    name: "imageDropPaste",

    addProseMirrorPlugins() {
      if (!upload) return [];
      const editor = this.editor;

      const insertImages = (files: FileList | null | undefined): boolean => {
        const images = Array.from(files ?? []).filter((file) =>
          file.type.startsWith("image/"),
        );
        if (images.length === 0) return false;
        for (const file of images) {
          void upload(file).then((src) => {
            editor.chain().focus().setImage({ src }).run();
          });
        }
        return true;
      };

      return [
        new Plugin({
          key: new PluginKey("imageDropPaste"),
          props: {
            handlePaste: (_view, event) =>
              insertImages(event.clipboardData?.files),
            handleDrop: (_view, event) =>
              insertImages(event.dataTransfer?.files),
          },
        }),
      ];
    },
  });
}
