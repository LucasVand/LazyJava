package state.serverState.paragraphGenerator;

import java.util.ArrayList;
import java.util.HashMap;

/**
 * ParagraphGenerator
 * 
 * @author Lucas Vanderwielen
 */
public class ParagraphGenerator {

    static HashMap<Integer, ArrayList<Paragraph>> paraList;

    public ParagraphGenerator() {
        if (ParagraphGenerator.paraList == null) {
            ParagraphGenerator.initList();
        }
    }

    public Paragraph getParagraph(int difficulty) {
        int clamped = difficulty > 9 ? 9 : difficulty;
        int index = (int) (Math.random() * (double) paraList.get(clamped).size());
        return paraList.get(clamped).get(index);
    }

    static void initList() {

        ParagraphGenerator.paraList = new HashMap<>();
        int i = 0;
        for (String chunck : list) {
            String[] paragraphs = chunck.split("`");
            ArrayList<Paragraph> list = new ArrayList<>();

            for (String para : paragraphs) {
                list.add(new Paragraph(para.strip(), i));
            }
            paraList.put(i, list);

            i++;
        }

    }

    static String dif1 = // difficulty 1
            """
                    The sun is up and the day begins in a slow and calm way. The light fills the sky and spreads across the ground as people wake and start to move through their day. The air feels cool and soft and the world seems quiet for a short time before the noise of the day starts to grow. Walking down the road feels easy and smooth as each step falls into a steady and simple pace that does not rush or push too hard.`
                    A small breeze moves through the trees and the leaves shift in a soft and steady way. The sky is clear and blue and the clouds are thin and far apart. Cars move down the road in slow lines and the sound fades as they pass by. People walk and talk and share small moments as the day begins to take shape and time moves on in a calm and gentle flow that feels easy to follow.`
                    A warm cup sits on a table and thin steam rises into the air in soft lines that slowly fade away. The room is still and the light comes in from the side and rests on the floor. The quiet space makes it easy to sit and think as time moves on without stress or noise and the day continues to build in a smooth and simple way.`
                    The road is wide and the path is clear as people move from place to place with no rush or strain. The sky stays bright and the light grows stronger as the hours pass. Each step feels light and easy and the world seems to move in a slow and steady rhythm that keeps everything in place and in time.`
                    A bird moves across the sky with quick wings and a smooth path that carries it far away. The sound is soft and fades as it goes. The view above stays open and wide and the light fills every space with a calm and even glow that makes the day feel steady and clear.`
                    Trees stand tall along the path and their leaves move with each soft breeze that passes by. The motion is slow and easy to see and adds a sense of life to the calm space around them. People pass by and take in the view as the day continues to unfold in a gentle and simple way.`
                    The air stays fresh and the sky holds its bright tone as time moves on. Small sounds rise and fall but never grow too loud or sharp. The world feels calm and safe and the pace of the day stays slow and easy to follow from start to end.`
                    As the day goes on the light shifts and moves across the ground and walls in long soft lines. The calm feeling stays and the world continues in its steady path until the time comes to rest and the day slowly comes to a close.
                    """;
    static String dif2 = // difficulty 2
            """
                    The morning begins with a soft spread of light that moves across the room and settles gently along the floor and walls. The air carries a cool and fresh feeling that makes the start of the day feel calm and clear. People begin to move through their routines with slow and steady motion as the quiet of early hours gradually gives way to the soft rhythm of daily life.`
                    Along the streets footsteps form a gentle pattern as people walk with purpose but without hurry. The sound of distant cars blends into the background and fades as it moves away. The sky above remains wide and open with thin clouds drifting slowly across its pale blue surface as the light continues to grow stronger.`
                    A warm drink rests near a window and releases a thin stream of steam that curls upward before fading into the still air. The scent is mild and comforting and adds to the sense of calm that fills the space. The quiet moment allows for reflection as time passes in a smooth and steady flow.`
                    Buildings catch the light and reflect it in soft tones that shift as the sun rises higher. The contrast between shadow and brightness creates depth across the scene and highlights the gradual movement of time throughout the morning hours.`
                    Trees along the sidewalk respond to each passing breeze with subtle motion that brings life to the otherwise still environment. Leaves shift and settle in a pattern that feels natural and continuous without interruption or sudden change.`
                    Voices rise and fall in quiet conversation as people pass one another and continue on their way. The sound remains gentle and never overwhelms the calm atmosphere that defines the early part of the day.`
                    Inside quiet spaces light moves slowly across surfaces marking the steady progression of time. Each small change adds to the sense of continuity that carries the day forward without disruption.`
                    The morning develops into a balanced rhythm where movement and stillness exist together. Each moment connects naturally to the next and creates a steady and predictable flow that defines the experience of the day.
                    """;
    static String dif3 = // difficulty 3
            """
                    The early hours of the day unfold with a gradual spread of light that reaches across open spaces and settles into quiet corners of the environment. The atmosphere carries a cool and composed feeling that encourages a slower pace and a more deliberate awareness of each passing moment. As the world begins to wake movement appears in small and steady forms that build naturally into the rhythm of the day.`
                    Pedestrians move along familiar paths with measured steps that create a consistent and almost patterned sound against the ground. This subtle rhythm blends with distant city noise to form a layered background that remains calm and unobtrusive. Above them the sky maintains a soft clarity with thin clouds drifting in slow and continuous motion.`
                    A cup of coffee rests near the edge of a table and releases a steady stream of warmth into the surrounding air. The rising steam forms delicate shapes that dissolve quickly and leave behind a faint trace of scent that adds depth to the quiet setting. This simple detail contributes to the overall sense of calm and focus present in the moment.`
                    Architectural forms begin to stand out as the angle of light shifts and reveals new layers of detail across surfaces. Reflections appear in glass and fade just as quickly while shadows stretch and contract in response to the changing position of the sun. These subtle transitions create a dynamic yet controlled visual experience.`
                    Natural elements such as trees and open patches of greenery introduce a gentle contrast to the structured environment. Their movement remains soft and continuous driven by light air currents that pass through without force or disruption.`
                    Conversation occurs in low tones and does not disturb the balance of the scene. Each voice contributes briefly before fading into the background allowing the overall atmosphere to remain stable and undisturbed.`
                    Interior spaces reflect similar patterns of light and motion as the day progresses. Shifting illumination across walls and surfaces provides a quiet indication of time passing without the need for sudden change or interruption.`
                    The progression of the morning is defined by its consistency and gradual development. Each element whether natural or constructed participates in a unified flow that maintains balance and clarity as the day continues forward.
                    """;
    static String dif4 = // difficulty 4
            """
                    The morning develops with a gradual increase in light that spreads across surfaces and defines the structure of the space. The atmosphere feels composed and balanced as movement begins to emerge in a steady and predictable manner that reflects the natural progression of time.`
                    Individuals move through the environment with a sense of purpose that remains calm and controlled. Footsteps create a consistent rhythm that blends into the broader background of the waking city without introducing disruption or intensity.`
                    A warm beverage rests nearby and releases a steady stream of heat into the surrounding air. The subtle motion of rising steam contributes to the quiet visual texture of the moment and reinforces the sense of stillness.`
                    The sky maintains a clear and open appearance with thin layers of cloud drifting slowly across its surface. Light interacts with these forms in a way that produces soft variations in brightness and tone.`
                    Architectural elements respond to the changing angle of sunlight as reflections and shadows adjust in gradual and continuous patterns that reveal depth and structure.`
                    Natural features such as trees and planted areas introduce movement that contrasts with the rigid forms of constructed spaces. This motion remains gentle and consistent without abrupt variation.`
                    Auditory elements remain controlled and subdued as distant sounds merge into a cohesive background that supports the calm environment rather than overwhelming it.`
                    The overall experience of the morning is defined by its stability and measured development as each component contributes to a unified and continuous flow of time.
                    """;
    static String dif5 = // difficulty 5
            """
                    The early phase of the day is characterized by a steady expansion of light that defines spatial relationships and highlights subtle variations across surfaces. The environment maintains a composed quality that encourages observation and deliberate movement.`
                    Pedestrian activity increases gradually as individuals navigate familiar routes with consistent pacing that contributes to a structured and predictable rhythm within the broader setting.`
                    A freshly prepared drink introduces a localized sense of warmth as rising vapor disperses into the surrounding air and dissolves without leaving a lasting trace.`
                    The sky presents a controlled arrangement of color and form as elongated clouds transition slowly across an otherwise stable and open background.`
                    Reflections across glass and polished materials create temporary visual effects that shift in response to the evolving position of the sun.`
                    Organic elements provide a contrasting layer of motion as leaves respond to subtle atmospheric changes with continuous and fluid movement.`
                    Ambient sound remains evenly distributed and restrained as it integrates multiple sources into a unified acoustic backdrop.`
                    The progression of the morning reflects a balance between motion and stillness where each transition occurs without disruption and contributes to a cohesive experience.
                    """;
    static String dif6 = // difficulty 6
            """
                    The morning interval unfolds through a gradual intensification of illumination that enhances spatial clarity and reveals layered structural detail throughout the environment. The atmosphere retains a composed and methodical character.`
                    Human activity integrates seamlessly into the setting as movement patterns align with established pathways and contribute to a coherent and uninterrupted flow.`
                    Thermal contrast is introduced through the presence of a warm beverage whose emitted vapor dissipates progressively into the surrounding air.`
                    The sky exhibits a restrained variation in tone as diffuse cloud formations migrate slowly across a stable visual field.`
                    Constructed surfaces interact dynamically with light as reflective and absorptive properties generate shifting patterns of brightness and shadow.`
                    Vegetation introduces a subtle kinetic element as external air currents produce continuous and non disruptive motion across leaves and branches.`
                    Acoustic conditions remain balanced with distributed sound sources contributing to a stable and unobtrusive auditory environment.`
                    Temporal progression is perceived through incremental transitions that maintain consistency and reinforce the structured continuity of the scene.
                    """;
    static String dif7 = // difficulty 7
            """
                    The transitional period of the morning is defined by a progressive amplification of ambient light that enhances perceptual depth and accentuates spatial organization across the environment. The resulting atmosphere promotes attentiveness and controlled engagement.`
                    Patterns of human circulation develop with increasing regularity as individuals adhere to established trajectories that reinforce systemic order within the urban framework.`
                    Localized thermal variation emerges through the presence of a heated beverage whose vaporization introduces a transient visual and sensory component.`
                    Atmospheric conditions remain stable with stratified cloud formations exhibiting slow and continuous displacement across the visible sky.`
                    Material surfaces demonstrate varied optical responses as reflective indices produce evolving interactions with incoming light.`
                    Biological elements contribute dynamic variability through sustained yet moderate motion influenced by environmental forces.`
                    Acoustic input remains harmonized as multiple sources integrate into a consistent and non intrusive soundscape.`
                    The cumulative progression of these elements establishes a coherent temporal sequence that advances without interruption or irregularity.
                    """;
    static String dif8 = // difficulty 8
            """
                    The initial segment of the day is characterized by a systematic escalation in luminosity that reveals complex spatial hierarchies and enhances perceptual resolution across both natural and constructed elements.`
                    Human behavioral patterns align with infrastructural design as movement flows adhere to predetermined pathways and maintain systemic efficiency.`
                    The introduction of a thermally active object generates localized atmospheric interaction as vapor diffusion occurs in a gradual and visually perceptible manner.`
                    Meteorological stability is maintained as diffuse cloud structures traverse the sky with minimal variation in velocity or density.`
                    Architectural materials exhibit differential reflective and absorptive characteristics that contribute to a continuously evolving visual composition.`
                    Organic components provide adaptive motion through consistent response to low intensity environmental stimuli.`
                    The acoustic environment sustains equilibrium through the integration of distributed sound sources into a balanced auditory field.`
                    Temporal continuity is reinforced through incremental and predictable transitions that preserve structural and perceptual coherence.
                    """;
    static String dif9 = // difficulty 9
            """
                    The early diurnal phase manifests through a calibrated augmentation of ambient luminosity that systematically enhances environmental legibility and reveals intricate spatial interdependencies across multiple structural layers.`
                    Anthropogenic movement patterns synchronize with established infrastructural systems thereby reinforcing operational continuity and spatial efficiency within the urban context.`
                    Localized thermodynamic activity is introduced via a heated liquid medium whose vapor phase transition produces transient yet observable atmospheric interaction.`
                    The atmospheric composition remains relatively stable as stratocumulus formations exhibit gradual lateral displacement without significant alteration in density or distribution.`
                    Built environments demonstrate complex photonic interactions as material properties influence reflectance and absorption coefficients under variable illumination conditions.`
                    Biotic elements contribute responsive motion patterns governed by external kinetic inputs that remain consistent in magnitude and direction.`
                    The acoustic field is characterized by a homogenized integration of distributed auditory stimuli resulting in a stable and non disruptive sensory baseline.`
                    Chronological progression is articulated through continuous and incremental transformations that preserve systemic equilibrium and perceptual consistency.
                    """;
    static String dif10 = // difficulty 10
            """
                    There is a subtle complexity to the way a morning unfolds in a well ordered space. Light does not simply appear but gradually asserts itself across surfaces, revealing textures and edges that were previously hidden in shadow. The process feels deliberate and measured, as though the environment itself is participating in a quiet and methodical transition from rest to activity.`
                    Urban environments reflect a layered history of intention and adaptation, where each structure contributes to a broader narrative of growth and revision. Glass and steel elements stand beside older materials, creating a contrast that highlights both innovation and persistence within the same visual field.`
                    The act of focusing attention has become increasingly valuable in a landscape filled with constant distraction. By directing awareness toward a single task or observation, it becomes possible to experience depth and clarity that would otherwise remain inaccessible amid continuous interruption.`
                    Sound within a city rarely disappears entirely but instead reorganizes itself into patterns that can either overwhelm or support concentration depending on their structure. A balanced arrangement of ambient noise can create a stable backdrop that allows thought to develop without resistance.`
                    Natural elements embedded within constructed environments provide an essential counterbalance to rigid design. The movement of leaves and the subtle variation of light across organic surfaces introduce unpredictability that enriches the overall sensory experience.`
                    Time progresses not through abrupt shifts but through a sequence of nearly imperceptible adjustments that accumulate into noticeable change. Recognizing these transitions requires patience and a willingness to observe details that are often overlooked.`
                    The interaction between light and material defines much of what is perceived within any space. Reflective surfaces amplify brightness while textured forms diffuse it, creating a dynamic interplay that evolves continuously throughout the day.`
                    In moments of stillness it becomes possible to perceive the underlying structure of an environment with greater clarity. Without urgency or distraction, each element reveals its role within a larger and more cohesive system.
                    """;

    static String[] list = { dif1, dif2, dif3, dif4, dif5, dif6, dif7, dif7, dif9, dif10 };

}
