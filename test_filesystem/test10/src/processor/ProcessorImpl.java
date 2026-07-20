package processor;

import javax.annotation.processing.*;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.*;
import javax.tools.Diagnostic;
import java.util.Set;

@SupportedAnnotationTypes("processor.Annot")
@SupportedSourceVersion(SourceVersion.RELEASE_25)
public class ProcessorImpl extends AbstractProcessor {

    @Override
    public boolean process(
            Set<? extends TypeElement> annotations,
            RoundEnvironment roundEnv) {

        processingEnv.getMessager().printMessage(
                Diagnostic.Kind.NOTE,
                "This message WILL show up in your terminal!");

        return true;
    }
}
